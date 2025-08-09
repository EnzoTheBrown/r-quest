use crate::loader::{Header, Request};
use anyhow::{Context, Result};
use colored::Colorize;
use reqwest::cookie::Jar;
use reqwest::{header::HeaderName, redirect, Client, Method};
use std::{collections::HashMap, sync::Arc};

fn pretty_json(s: &str) -> String {
    serde_json::from_str::<serde_json::Value>(s)
        .map(|v| serde_json::to_string_pretty(&v).unwrap_or_else(|_| s.to_string()))
        .unwrap_or_else(|_| s.to_string())
}

pub async fn run_single_request(
    base_url: &str,
    project: &str,
    env: &str,
    request: &Request,
) -> Result<()> {
    let jar = Arc::new(Jar::default());
    let client = Client::builder()
        .user_agent("qwest/0.2 (rust-cli-http)")
        .cookie_provider(jar)
        .redirect(redirect::Policy::limited(10))
        .build()
        .context("building reqwest client")?;

    execute(&client, base_url, project, env, request).await?;
    Ok(())
}

async fn execute(
    client: &Client,
    base_url: &str,
    project: &str,
    env: &str,
    req: &Request,
) -> Result<()> {
    let mut vars = crate::db::load_vars(project, env).unwrap_or_default();

    if let Some(code) = &req.pre_script {
        let mut senv = crate::script::ScriptEnv {
            vars: &mut vars,
            status: None,
            headers: None,
            data: None,
            project: project.to_string(),
            env: env.to_string(),
        };
        crate::script::run_script(code, &mut senv)?;
    }

    let url = format!("{}{}", base_url, req.path);
    println!("{}", format!("→ {} {}", req.method, url).bold());

    let method =
        Method::from_bytes(req.method.as_bytes()).context("invalid HTTP method in config")?;
    let mut builder = client.request(method, &url);

    let mut content_type_form = false;
    for Header { key, value } in &req.headers {
        if key.eq_ignore_ascii_case("content-type")
            && value.eq_ignore_ascii_case("application/x-www-form-urlencoded")
        {
            content_type_form = true;
        }
        builder = builder.header(HeaderName::from_bytes(key.as_bytes())?, value);
    }

    if let Some(params) = &req.params {
        builder = builder.query(&params.as_object().unwrap_or(&serde_json::Map::new()));
    }

    if let Some(body) = &req.body {
        if content_type_form {
            let obj = body.as_object().context("form body must be JSON object")?;
            let form: HashMap<String, String> = obj
                .iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_owned()))
                .collect();
            builder = builder.form(&form);
        } else {
            builder = builder.json(body);
        }
    }

    // Send
    let resp = builder.send().await.context("HTTP send failed")?;
    let status = resp.status();
    let headers_map: HashMap<String, String> = resp
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let text = resp
        .text()
        .await
        .unwrap_or_else(|_| "<non-utf8 body>".into());

    let status_str = status.as_u16().to_string();
    let colored_status = if status.is_success() {
        status_str.green().bold()
    } else if status.is_client_error() {
        status_str.yellow().bold()
    } else if status.is_server_error() {
        status_str.red().bold()
    } else {
        status_str.normal()
    };
    println!(
        "{} {} {}",
        "←".bold(),
        colored_status,
        status.canonical_reason().unwrap_or("")
    );

    for (k, v) in &headers_map {
        println!("{}: {}", k.dimmed(), v);
    }
    println!();
    println!("{}\n", pretty_json(&text));

    if let Some(code) = req.test_script.as_ref().or(req.spell.as_ref()) {
        let mut senv = crate::script::ScriptEnv {
            vars: &mut vars,
            status: Some(status.as_u16() as i64),
            headers: Some(headers_map),
            data: serde_json::from_str::<serde_json::Value>(&text).ok(),
            project: project.to_string(),
            env: env.to_string(),
        };
        crate::script::run_script(code, &mut senv)?;
        println!("{}", "✓ tests passed".green().bold());
    }

    crate::db::upsert_vars(project, env, &vars)?;

    Ok(())
}
