use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::Path;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    #[serde(default)]
    pub admin_token: Option<String>,

    #[serde(default)]
    pub core: CoreConfig,

    #[serde(default)]
    pub debug: bool,

    #[serde(default)]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CoreConfig {
    #[serde(default = "default_core_timeout")]
    pub timeout: String,

    #[serde(default = "default_core_retries")]
    pub retries: i32,
}

const DEFAULT_CORE_TIMEOUT: &str = "2m";
const DEFAULT_CORE_RETRIES: i32 = 3;

fn default_core_timeout() -> String {
    DEFAULT_CORE_TIMEOUT.to_string()
}
fn default_core_retries() -> i32 {
    DEFAULT_CORE_RETRIES
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            timeout: default_core_timeout(),
            retries: default_core_retries(),
        }
    }
}

pub fn load_config(path: &Path) -> Config {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Config::default(),
    };

    match serde_json::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("warning: failed to parse {}: {}", path.display(), e);
            Config::default()
        }
    }
}

pub fn apply_env_overrides(app_name: &str, config: &mut Config) {
    let prefix = format!("{}_", app_name.to_uppercase());

    let current = serde_json::to_value(&*config).unwrap_or(serde_json::Value::Null);
    let mut merged = current.clone();
    let mut leaf_keys = BTreeSet::new();
    leaf_keys_recursive(&current, "", &mut leaf_keys);

    for key in &leaf_keys {
        let env_key = format!("{}{}", prefix, key.to_uppercase().replace('-', "_"));
        if let Ok(val) = std::env::var(&env_key) {
            set_json_path(&mut merged, key, env_string_to_value(&val));
        }
    }

    if let Ok(updated) = serde_json::from_value(merged) {
        *config = updated;
    }
}

fn leaf_keys_recursive(value: &serde_json::Value, prefix: &str, out: &mut BTreeSet<String>) {
    if let serde_json::Value::Object(map) = value {
        for (k, v) in map {
            let key = if prefix.is_empty() {
                k.clone()
            } else {
                format!("{}-{}", prefix, k)
            };
            if v.is_object() {
                leaf_keys_recursive(v, &key, out);
            } else {
                out.insert(key);
            }
        }
    }
}

fn set_json_path(value: &mut serde_json::Value, path: &str, val: serde_json::Value) {
    let map = match value.as_object_mut() {
        Some(m) => m,
        None => return,
    };

    let mut parts = path.splitn(2, '-');
    let first = parts.next().unwrap_or(path);

    if let Some(rest) = parts.next()
        && !map.contains_key(path)
    {
        let entry = map
            .entry(first.to_string())
            .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
        return set_json_path(entry, rest, val);
    }

    map.insert(path.to_owned(), val);
}

fn env_string_to_value(val: &str) -> serde_json::Value {
    match val {
        "true" | "yes" | "1" => return serde_json::Value::Bool(true),
        "false" | "no" | "0" => return serde_json::Value::Bool(false),
        _ => {}
    }
    if let Ok(n) = val.parse::<i64>() {
        return serde_json::Value::Number(n.into());
    }
    if let Ok(f) = val.parse::<f64>()
        && let Some(n) = serde_json::Number::from_f64(f)
    {
        return serde_json::Value::Number(n);
    }
    serde_json::Value::String(val.to_owned())
}

#[macro_export]
macro_rules! config_defaults {
    ($ty:ty { $($field:ident => ($single:ident)),+ $(,)? }) => {
        impl $ty {
            pub fn apply_config_defaults(&mut self, config: &$crate::config::Config) {
                $(
                    if self.$field.is_none() {
                        self.$field = config.$single.clone();
                    }
                )+
            }
        }
    };
    ($ty:ty { $($field:ident => ($first:ident $(, $rest:ident)+)),+ $(,)? }) => {
        impl $ty {
            pub fn apply_config_defaults(&mut self, config: &$crate::config::Config) {
                $(
                    if self.$field.is_none() {
                        self.$field = Some(config.$first $(.$rest)+ .clone());
                    }
                )+
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn env_string_to_value_parses_booleans() {
        assert_eq!(env_string_to_value("true"), json!(true));
        assert_eq!(env_string_to_value("yes"), json!(true));
        assert_eq!(env_string_to_value("1"), json!(true));
        assert_eq!(env_string_to_value("false"), json!(false));
        assert_eq!(env_string_to_value("0"), json!(false));
    }

    #[test]
    fn env_string_to_value_parses_numbers() {
        assert_eq!(env_string_to_value("42"), json!(42));
        assert_eq!(env_string_to_value("3.14"), json!(3.14));
    }

    #[test]
    fn env_string_to_value_falls_back_to_string() {
        assert_eq!(env_string_to_value("hello"), json!("hello"));
        assert_eq!(env_string_to_value("2m"), json!("2m"));
    }

    #[test]
    fn set_json_path_creates_nested_objects() {
        let mut v = json!({});
        set_json_path(&mut v, "core-timeout", json!("5m"));
        assert_eq!(v, json!({"core": {"timeout": "5m"}}));
    }

    #[test]
    fn set_json_path_overwrites_existing() {
        let mut v = json!({"core": {"timeout": "2m"}});
        set_json_path(&mut v, "core-timeout", json!("5m"));
        assert_eq!(v, json!({"core": {"timeout": "5m"}}));
    }

    #[test]
    fn leaf_keys_collects_nested_paths() {
        let v = json!({"admin-token": "x", "core": {"timeout": "2m", "retries": 3}});
        let mut keys = BTreeSet::new();
        leaf_keys_recursive(&v, "", &mut keys);
        assert!(keys.contains("admin-token"));
        assert!(keys.contains("core-timeout"));
        assert!(keys.contains("core-retries"));
    }

    #[test]
    fn apply_env_overrides_sets_matching_env() {
        let mut config = Config::default();
        // SAFETY: single-threaded test; no concurrent readers of this var.
        unsafe { std::env::set_var("MYCLI_CORE_TIMEOUT", "9m") };
        apply_env_overrides("mycli", &mut config);
        unsafe { std::env::remove_var("MYCLI_CORE_TIMEOUT") };
        assert_eq!(config.core.timeout, "9m");
    }
}
