use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    #[serde(default, skip_serializing_if = "Option::is_none")]
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

/// Result of loading a config file.
pub struct LoadedConfig {
    pub config: Config,
    pub raw: Option<serde_json::Value>,
}

/// Load a JSON config file. If the file doesn't exist or is invalid,
/// returns defaults and logs a warning to stderr.
pub fn load_config(path: &Path) -> LoadedConfig {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return LoadedConfig { config: Config::default(), raw: None },
    };

    let raw: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("warning: failed to parse {}: {}", path.display(), e);
            return LoadedConfig { config: Config::default(), raw: None };
        }
    };

    let typed: Config = match serde_json::from_value(raw.clone()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("warning: config in {} has unexpected structure: {}", path.display(), e);
            Config::default()
        }
    };

    LoadedConfig { config: typed, raw: Some(raw) }
}

/// Detect duplicate config keys. Returns an error listing the first duplicate.
pub fn detect_duplicates(value: &serde_json::Value) -> Result<()> {
    let mut seen = BTreeMap::new();

    fn walk(
        value: &serde_json::Value,
        prefix: &str,
        dot_prefix: &str,
        seen: &mut BTreeMap<String, String>,
    ) -> Result<()> {
        match value {
            serde_json::Value::Object(map) => {
                for (k, v) in map {
                    let flat_key = if prefix.is_empty() {
                        k.clone()
                    } else {
                        format!("{}-{}", prefix, k)
                    };
                    let dot_key = if dot_prefix.is_empty() {
                        k.clone()
                    } else {
                        format!("{}.{}", dot_prefix, k)
                    };
                    if v.is_object() {
                        walk(v, &flat_key, &dot_key, seen)?;
                    } else if let Some(prev) = seen.insert(flat_key.clone(), dot_key.clone()) {
                        return Err(anyhow!(
                            "both {:?} and {:?} map to config key {:?}",
                            prev,
                            dot_key,
                            flat_key
                        ));
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    walk(value, "", "", &mut seen)
}

/// Apply env var overrides to a config, using `{APPNAME}_` as prefix.
///
/// For each leaf field in the config, the env var name is computed as
/// `{APP}_{FLAT_KEY}` (e.g. `MAX_CORE_TIMEOUT` → `core-timeout`).
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

/// Recursively collect all leaf paths in a JSON value.
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

/// Set a value at a flat key path (e.g. `"core-timeout"`) in a JSON tree.
///
/// If the full flat key already exists as a single key at the current level,
/// it is treated as atomic (e.g. `"dry-run"`). Otherwise the first hyphen
/// separates nesting levels (e.g. `"core-timeout"` → `["core"]["timeout"]`).
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

/// Best-effort conversion of an env-var string into a typed JSON value.
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
