use crate::cli::ConfigCommands;
use crate::config::Config;
use anyhow::Result;
use std::path::Path;

pub fn execute(cmd: &ConfigCommands, _cfg: &Config, config_path: &Path) -> Result<()> {
    match cmd {
        ConfigCommands::Init { force } => {
            if config_path.exists() && !force {
                eprintln!("config file already exists at {}", config_path.display());
                return Ok(());
            }
            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let default = Config::default();
            let json = serde_json::to_string_pretty(&default)?;
            std::fs::write(config_path, json)?;
            println!("config file created at {}", config_path.display());
        }
        ConfigCommands::Show => {
            let content = std::fs::read_to_string(config_path).unwrap_or_else(|_| "{}".to_string());
            println!("{}", content);
        }
        ConfigCommands::Path => {
            println!("{}", config_path.display());
        }
    }
    Ok(())
}
