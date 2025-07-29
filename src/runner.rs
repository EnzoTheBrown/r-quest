use crate::loader::{Config, Request, Header};
use anyhow::{Context, Result};
use reqwest::{header::HeaderName, Client, Method};
use rhai::serde::to_dynamic;
use rhai::{Dynamic, Engine, Map as RhaiMap, Scope};
use serde_json::{Map as JsonMap, Value};
use std::collections::HashMap;
use std::fs;

pub async fn run_config(cfg: &Config) -> Result<()> {
    let refs: Vec<&Request> = cfg.requests.iter().collect();
    run_requests(&cfg.api.base_url, &refs).await
}

pub async fn run_single_request(base_url: &str, request: &Request) -> Result<()> {
    let client = Client::builder()
        .user_agent("rust-cli-httpclient/0.1")
        .build()
        .context("building reqwest client")?;
    execute(&client, base_url, request).await?;
    Ok(())
}

pub async fn run_requests(base_url: &str, requests: &[&Request]) -> Result<()> {
    let client = Client::builder()
        .user_agent("rust-cli-httpclient/0.1")
        .build()
        .context("building reqwest client")?;
    for req in requests {
        execute(&client, base_url, req).await?;
    }
    Ok(())
}

fn rhai_dynamic_to_string_map(value: Dynamic) -> Result<JsonMap<String, Value>> {
    if !value.is::<RhaiMap>() {
        return Err(anyhow::anyhow!("Expected a Rhai map at top level"));
    }
    let rhai_map = value.cast::<RhaiMap>();
    let mut map = JsonMap::new();
    for (k, v) in rhai_map {
        if v.is::<String>() {
            map.insert(k.to_string(), Value::String(v.cast::<String>()));
        } else {
            return Err(anyhow::anyhow!(
                "Value for key '{}' is not a string",
                k.to_string()
            ));
        }
    }
    Ok(map)
}

pub async fn run_script(script: String, response: &str) -> Result<()> {
    let parsed: Value = serde_json::from_str(response).context("HTTP body is not valid JSON")?;
    let engine = Engine::new();
    let mut scope = Scope::new();
    scope.push_dynamic("data", to_dynamic(parsed)?);
    let dynamic = engine
        .eval_with_scope::<Dynamic>(&mut scope, &script)
        .map_err(|e| anyhow::anyhow!("Rhai error: {e}"))?;
    let map = rhai_dynamic_to_string_map(dynamic)?;
    save_result_in_memomy(map).await?;
    Ok(())
}

async fn save_result_in_memomy(result: JsonMap<String, Value>) -> Result<()> {
    let mut path = dirs::home_dir().context("cannot determine home directory")?;
    path.push(".config/quest");
    fs::create_dir_all(&path)?;
    path.push("mem.json");

    fs::write(&path, serde_json::to_vec_pretty(&Value::Object(result))?)
        .context("writing mem.json")?;
    Ok(())
}
async fn execute(client: &Client, base_url: &str, req: &Request) -> Result<()> {
    let url = format!("{}{}", base_url, req.path);
    println!("Executing request: {}", url);

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

    if let Some(body) = &req.body {
        if content_type_form {
            let obj = body
                .as_object()
                .context("form body must be a JSON object")?;
            let form: HashMap<String, String> = obj
                .iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_owned()))
                .collect();
            builder = builder.form(&form);
        } else {
            builder = builder.json(body);
        }
    }

    if let Some(params) = &req.params {
        builder = builder.query(&params.as_object().unwrap_or(&serde_json::Map::new()));
    }

    let resp = builder.send().await.context("HTTP send failed")?;
    let status = resp.status();
    let text = resp
        .text()
        .await
        .unwrap_or_else(|_| "<non‑utf8 body>".into());

    println!("{} {} → {}", req.method, req.path, status);
    println!("{text}\n");

    if let Some(script) = &req.script {
        run_script(script.clone(), &text).await?;
    }

    Ok(())
}
