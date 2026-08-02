use crate::cli::ConfigCommands;
use crate::config::Config;
use anyhow::Result;
use std::path::Path;

fn set_nested(m: &mut serde_json::Value, key: &str, value: serde_json::Value) -> Result<()> {
    if let Some((first, rest)) = key.split_once('.') {
        let map = m.as_object_mut().ok_or_else(|| anyhow::anyhow!("cannot set key {key:?}: expected object"))?;
        if !map.contains_key(first) {
            map.insert(first.to_string(), serde_json::Value::Object(serde_json::Map::new()));
        }
        if let Some(sub) = map.get_mut(first) {
            set_nested(sub, rest, value)?;
        }
        Ok(())
    } else if let Some(obj) = m.as_object_mut() {
        obj.insert(key.to_string(), value);
        Ok(())
    } else {
        anyhow::bail!("cannot set key {key:?}: root value is not an object");
    }
}

fn unset_nested(m: &mut serde_json::Value, key: &str) -> Result<()> {
    if let Some((first, rest)) = key.split_once('.') {
        if let Some(sub) = m.as_object_mut().and_then(|obj| obj.get_mut(first)) {
            unset_nested(sub, rest)?;
            if sub.as_object().is_some_and(|o| o.is_empty()) {
                m.as_object_mut().map(|obj| obj.remove(first));
            }
        }
        Ok(())
    } else if let Some(obj) = m.as_object_mut() {
        obj.remove(key);
        Ok(())
    } else {
        Ok(())
    }
}

fn read_config(path: &Path) -> Result<serde_json::Value> {
    match std::fs::read_to_string(path) {
        Ok(s) => match serde_json::from_str(&s) {
            Ok(v) => Ok(v),
            Err(e) => {
                eprintln!("warning: failed to parse {}: {}", path.display(), e);
                Ok(serde_json::Value::Object(serde_json::Map::new()))
            }
        },
        Err(_) => Ok(serde_json::Value::Object(serde_json::Map::new())),
    }
}

fn write_config(path: &Path, value: &serde_json::Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(value)?;
    std::fs::write(path, data)?;
    Ok(())
}

pub fn execute(cmd: ConfigCommands, _cfg: &Config, config_path: &Path) -> Result<()> {
    match cmd {
        ConfigCommands::Init(init_args) => {
            if config_path.exists() && !init_args.force {
                anyhow::bail!("config file already exists at {}", config_path.display());
            }
            write_config(config_path, &serde_json::Value::Object(serde_json::Map::new()))?;
            println!("Config file created at {}", config_path.display());
        }
        ConfigCommands::Set(set_args) => {
            let path = set_args.config_file.as_ref().map(Path::new).unwrap_or(config_path);
            let mut config = read_config(path)?;
            set_nested(&mut config, &set_args.key, serde_json::Value::String(set_args.value.clone()))?;
            write_config(path, &config)?;
            println!("{} = {}", set_args.key, set_args.value);
        }
        ConfigCommands::Unset(unset_args) => {
            let path = unset_args.config_file.as_ref().map(Path::new).unwrap_or(config_path);
            let mut config = read_config(path)?;
            unset_nested(&mut config, &unset_args.key)?;
            write_config(path, &config)?;
            println!("{}: unset", unset_args.key);
        }
        ConfigCommands::Path => {
            if !config_path.exists() {
                println!("{} (does not exist)", config_path.display());
            } else {
                println!("{}", config_path.display());
            }
        }
        ConfigCommands::Show => {
            let path = config_path;
            if !path.exists() {
                eprintln!("warning: config file not found at {}", path.display());
                println!("{{}}");
            } else {
                let config = read_config(path)?;
                println!("{}", serde_json::to_string_pretty(&config)?);
            }
        }
        ConfigCommands::Edit => {
            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            if !config_path.exists() {
                std::fs::write(config_path, b"{}\n")?;
            }
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
            let status = std::process::Command::new(&editor).arg(config_path).status()?;
            if !status.success() {
                anyhow::bail!("editor exited with error");
            }
        }
    }
    Ok(())
}
