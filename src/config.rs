use std::path::PathBuf;

pub type Config = serde_json::Value;

pub fn config_path() -> PathBuf {
    if let Ok(cf) = std::env::var("MAX_CONFIG_FILE")
        && !cf.is_empty()
    {
        return PathBuf::from(cf);
    }
    if PathBuf::from("max.json").exists() {
        return PathBuf::from("max.json");
    }
    if let Some(config_dir) = dirs::config_dir() {
        return config_dir.join("max").join("max.json");
    }
    PathBuf::from("max.json")
}

pub fn load_config() -> Config {
    let path = config_path();
    match std::fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
        Err(_) => serde_json::Value::Object(serde_json::Map::new()),
    }
}
