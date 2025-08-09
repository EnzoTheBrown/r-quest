use serde_json::Value;
use std::{collections::HashMap, fs, path::PathBuf};

fn load_json_env() -> anyhow::Result<HashMap<String, String>> {
    let mut path = dirs::home_dir().unwrap_or(PathBuf::from("/"));
    path.push(".config/qwest/mem.json");

    if !path.exists() {
        return Ok(HashMap::new());
    }
    let contents = fs::read_to_string(path)?;
    let json: Value = serde_json::from_str(&contents)?;
    let map = json
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("mem.json should be a JSON object"))?
        .iter()
        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
        .collect();
    Ok(map)
}

pub fn load_env() -> anyhow::Result<HashMap<String, String>> {
    let mut vars = load_json_env()?;
    for (key, val) in std::env::vars() {
        vars.insert(key, val);
    }
    Ok(vars)
}
