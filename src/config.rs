use std::path::PathBuf;

pub type Config = serde_json::Value;

pub fn config_path() -> PathBuf {
    let env_override = std::env::var("MAX_CONFIG_FILE").ok();
    let has_local = PathBuf::from("max.json").exists();
    resolve_config_path(env_override, has_local, dirs::config_dir())
}

fn resolve_config_path(
    env_override: Option<String>,
    has_local: bool,
    config_dir: Option<PathBuf>,
) -> PathBuf {
    if let Some(cf) = env_override.filter(|v| !v.is_empty()) {
        return PathBuf::from(cf);
    }
    if has_local {
        return PathBuf::from("max.json");
    }
    if let Some(config_dir) = config_dir {
        return config_dir.join("max").join("max.json");
    }
    PathBuf::from("max.json")
}

pub fn load_config() -> Config {
    let path = config_path();
    match std::fs::read_to_string(&path) {
        Ok(s) => match serde_json::from_str(&s) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("warning: failed to parse {}: {}", path.display(), e);
                serde_json::Value::Object(serde_json::Map::new())
            }
        },
        Err(_) => serde_json::Value::Object(serde_json::Map::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn env_override_wins() {
        let p = resolve_config_path(
            Some("/tmp/custom.json".to_string()),
            true,
            Some(PathBuf::from("/home/u/.config")),
        );
        assert_eq!(p, PathBuf::from("/tmp/custom.json"));
    }

    #[test]
    fn empty_env_override_ignored() {
        let p = resolve_config_path(Some(String::new()), false, None);
        assert_eq!(p, PathBuf::from("max.json"));
    }

    #[test]
    fn local_wins_over_xdg() {
        let p = resolve_config_path(None, true, Some(PathBuf::from("/home/u/.config")));
        assert_eq!(p, PathBuf::from("max.json"));
    }

    #[test]
    fn xdg_fallback() {
        let p = resolve_config_path(None, false, Some(PathBuf::from("/home/u/.config")));
        assert_eq!(p, PathBuf::from("/home/u/.config/max/max.json"));
    }

    #[test]
    fn final_fallback() {
        let p = resolve_config_path(None, false, None);
        assert_eq!(p, PathBuf::from("max.json"));
    }
}
