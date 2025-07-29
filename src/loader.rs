use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, fs};
use regex::Regex;
use std::path::PathBuf;


#[derive(Debug, Deserialize)]
pub struct Config {
    pub api: Api,
    #[serde(rename = "request")]
    pub requests: Vec<Request>,
}

#[derive(Debug, Deserialize)]
pub struct Api {
    pub name: String,
    pub description: Option<String>,
    pub base_url: String,
}

#[derive(Debug, Deserialize)]
pub struct Header {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct Request {
    pub name: String,
    pub method: String,
    pub path: String,

    #[serde(default, rename = "header")]
    pub headers: Vec<Header>,

    #[serde(default, deserialize_with = "json_string_opt")]
    pub body: Option<Value>,

    #[serde(default, deserialize_with = "json_string_opt")]
    pub params: Option<Value>,

    pub script: Option<String>,
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
    let re = Regex::new(r"\$\{([a-zA-Z0-9_]+)\}")?;
    let mut vars = load_json_env()?;
    println!("Loaded environment variables: {:?}", vars);

    let mut out = String::with_capacity(raw.len());
    let mut last = 0;

    for caps in re.captures_iter(raw) {
        println!("Found placeholder: {:?}", &caps[0]);
        let m = caps.get(0).unwrap();
        let key = &caps[1];

        if let Some(val) = vars.get(key) {
            out.push_str(&raw[last..m.start()]);
            out.push_str(val);
            last = m.end();
        }
    }
    out.push_str(&raw[last..]);
    Ok(out)
}

fn load_json_env() -> anyhow::Result<HashMap<String, String>> {
    let mut path = dirs::home_dir().unwrap_or(PathBuf::from("/"));
    path.push(".config/quest/mem.json");

    if !path.exists() {
        return Ok(HashMap::new());
    }

    let contents = fs::read_to_string(path)?;
    let json: Value = serde_json::from_str(&contents)?;

    let map = json.as_object()
        .ok_or_else(|| anyhow::anyhow!("mem.json should be a JSON object"))?
        .iter()
        .filter_map(|(k, v)| {
            v.as_str().map(|s| (k.clone(), s.to_string()))
        })
        .collect();

    Ok(map)
}
