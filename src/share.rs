use anyhow::{anyhow, Context, Result};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde_json::json;

pub async fn share(base_url: &str, qwest_name: &str, content: &str) -> Result<String> {
    let client = Client::builder()
        .user_agent("Qwest/0.1")
        .build()
        .context("building reqwest client")?;

    let resp = client
        .post(base_url)
        .json(&json!({ "name": qwest_name, "value": content }))
        .send()
        .await
        .context("sending POST /config")?;

    if resp.status() == StatusCode::CREATED {
        #[derive(Deserialize)]
        struct Created {
            id: String,
        }

        let Created { id } = resp.json::<Created>().await.context("parsing JSON body")?;

        Ok(id)
    } else {
        Err(anyhow!(
            "unexpected status {} from {}",
            resp.status(),
            base_url
        ))
    }
}
