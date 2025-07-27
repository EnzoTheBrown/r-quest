use regex::Regex;
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, fs};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub api: Api,
    #[serde(rename = "request")]
    pub requests: Vec<Request>,
}

#[derive(Debug, Deserialize)]
pub struct Api {
    pub base_url: String,
}

#[derive(Debug, Deserialize)]
pub struct Request {
    pub name: String,
    pub method: String,
    pub path: String,

    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,

    #[serde(default, deserialize_with = "json_string_opt")]
    pub body: Option<Value>,

    #[serde(default, deserialize_with = "json_string_opt")]
    pub params: Option<Value>,

    #[serde(default, deserialize_with = "json_string_opt")]
    pub script: Option<Value>,
}

fn json_string_opt<'de, D>(de: D) -> Result<Option<Value>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt {
        Some(raw) => serde_json::from_str(&raw)
            .map(Some)
            .map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

pub fn load_config(path: &str) -> anyhow::Result<Config> {
    let raw = fs::read_to_string(path)?;
    let expanded = expand_placeholders(&raw)?;
    Ok(toml::from_str(&expanded)?)
}

fn expand_placeholders(raw: &str) -> anyhow::Result<String> {
    let re = Regex::new(r"\$\{([A-Z0-9_]+)\}")?;
    let mut out = String::with_capacity(raw.len());
    let mut last = 0;

    for caps in re.captures_iter(raw) {
        let m = caps.get(0).unwrap();
        let key = &caps[1];
        let maybe_val = std::env::var(key);

        if maybe_val.is_err() {
            continue;
        } else {
            let val = maybe_val.unwrap();
            if val.is_empty() {
                continue;
            }
            out.push_str(&raw[last..m.start()]);
            out.push_str(&val);
            last = m.end();
        }
    }
    out.push_str(&raw[last..]);
    Ok(out)
}
