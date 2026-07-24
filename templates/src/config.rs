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
