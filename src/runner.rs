use crate::loader::{Config, Request};
use anyhow::{Context, Result};
use reqwest::{header::HeaderName, Client, Method};
use serde_json::Value;

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

async fn execute(client: &Client, base_url: &str, req: &Request) -> Result<()> {
    let url = format!(
        "{}{}",
        base_url.trim_end_matches('/'),
        if req.path.starts_with('/') { "" } else { "/" },
    ) + &req.path;

    let method =
        Method::from_bytes(req.method.as_bytes()).context("invalid HTTP method in config")?;

    let mut builder = client.request(method, &url);

    if let Some(hdrs) = &req.headers {
        for (k, v) in hdrs {
            builder = builder.header(HeaderName::from_bytes(k.as_bytes())?, v);
        }
    }
    if let Some(json) = &req.body {
        builder = builder.json(json);
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
    Ok(())
}
