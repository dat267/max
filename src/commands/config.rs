use crate::cli::ConfigCommands;
use crate::config::Config;
use anyhow::{Context, Result};
use std::path::Path;

pub fn execute(
    cmd: &ConfigCommands,
    config: &Config,
    config_path: &Path,
) -> Result<()> {
    match cmd {
        ConfigCommands::Init { force } => cmd_init(*force, config_path),
        ConfigCommands::Show { json } => cmd_show(*json, config, config_path),
        ConfigCommands::Path => cmd_path(config_path),
    }
}

fn cmd_init(force: bool, config_path: &Path) -> Result<()> {
    if config_path.exists() && !force {
        eprintln!("Config file already exists at {}", config_path.display());
        eprintln!("Use --force to overwrite.");
        return Ok(());
    }

    let default_config = Config::default();
    let json = serde_json::to_string_pretty(&default_config)
        .context("failed to serialize default config")?;

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .context("failed to create config directory")?;
    }

    std::fs::write(config_path, &json)
        .context("failed to write config file")?;

    println!("Initialized config file at {}", config_path.display());
    Ok(())
}

fn cmd_show(json: bool, config: &Config, config_path: &Path) -> Result<()> {
    if !config_path.exists() {
        eprintln!("Config file not found at {}", config_path.display());
        eprintln!("Run 'init' to create a default config.");
        return Ok(());
    }

    if json {
        let output = serde_json::to_string_pretty(config)
            .context("failed to serialize config")?;
        println!("{}", output);
    } else {
        let content = std::fs::read_to_string(config_path)
            .context("failed to read config file")?;
        print!("{}", content);
    }

    Ok(())
}

fn cmd_path(config_path: &Path) -> Result<()> {
    println!("{}", config_path.display());
    Ok(())
}
