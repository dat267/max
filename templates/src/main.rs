mod cli;
mod commands;
mod config;

use anyhow::{Context, Result};
use clap::Parser;
use std::path::{Path, PathBuf};

use cli::{Cli, Commands, DEFAULT_APP_NAME};
use config::Config;

fn main() -> Result<()> {
    let app_name = resolve_app_name();

    let config_path = resolve_config_path(&app_name);
    let mut cfg = config::load_config(&config_path);

    config::apply_env_overrides(&app_name, &mut cfg);

    let cli = Cli::parse();

    if cli.verbose {
        eprintln!("app:      {}", app_name);
        eprintln!("config:   {}", config_path.display());
    }

    run_command(cli, &cfg, &config_path)
}

fn resolve_app_name() -> String {
    let name = std::env::args()
        .next()
        .and_then(|p| {
            PathBuf::from(p)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
        })
        .unwrap_or_default();

    if matches!(name.as_str(), "" | "main" | "app")
        || name.starts_with("go-build")
        || name.ends_with(".test")
    {
        DEFAULT_APP_NAME.to_string()
    } else {
        name
    }
}

fn resolve_config_path(app_name: &str) -> PathBuf {
    let env_key = format!("{}_CONFIG_FILE", app_name.to_uppercase());
    if let Ok(path) = std::env::var(&env_key)
        && !path.is_empty()
    {
        return PathBuf::from(path);
    }

    let local = PathBuf::from(format!("{app_name}.json"));
    if local.exists() {
        return local;
    }

    if let Some(config_dir) = dirs::config_dir() {
        return config_dir.join(app_name).join(format!("{app_name}.json"));
    }

    PathBuf::from(format!("{app_name}.json"))
}

fn run_command(cli: Cli, cfg: &Config, config_path: &Path) -> Result<()> {
    match cli.command {
        Commands::Config(cmd) => commands::config::execute(&cmd, cfg, config_path)
            .context("config command failed")?,
        Commands::Greet(mut args) => {
            args.apply_config_defaults(cfg);
            commands::greet::execute(&args, cfg).context("greet command failed")?
        }
    }
    Ok(())
}
