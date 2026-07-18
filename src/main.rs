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

    let config_file_from_args = scan_config_file_arg();
    let config_path = resolve_config_path(&app_name, config_file_from_args.as_deref());

    let loaded = config::load_config(&config_path);

    if let Some(ref raw_value) = loaded.raw
        && let Err(dup_err) = config::detect_duplicates(raw_value)
    {
        eprintln!(
            "error: duplicate config keys in {}: {}. Fix the file and try again.",
            config_path.display(),
            dup_err
        );
        std::process::exit(1);
    }

    let mut cfg = loaded.config;
    config::apply_env_overrides(&app_name, &mut cfg);

    let cli = Cli::parse();

    if cli.verbose {
        eprintln!("app:      {}", app_name);
        eprintln!("config:   {}", config_path.display());
        eprintln!("debug:    {}", cfg.debug);
        eprintln!("dry-run:  {}", cfg.dry_run);
    }

    if cli.verbose && cfg.dry_run {
        eprintln!("[dry-run] DRY-RUN is enabled — no destructive actions will be taken");
    }

    run_command(&cli, &cfg, &config_path)
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

    if matches!(
        name.as_str(),
        "" | "main" | "app"
    ) || name.starts_with("go-build")
        || name.ends_with(".test")
    {
        DEFAULT_APP_NAME.to_string()
    } else {
        name
    }
}

fn scan_config_file_arg() -> Option<String> {
    let mut args = std::env::args().peekable();
    while let Some(arg) = args.next() {
        if arg == "--config-file" {
            if let Some(val) = args.next()
                && !val.starts_with('-')
            {
                return Some(val);
            }
        } else if let Some(val) = arg.strip_prefix("--config-file=") {
            return Some(val.to_owned());
        }
    }
    None
}

fn resolve_config_path(app_name: &str, from_args: Option<&str>) -> PathBuf {
    if let Some(path) = from_args {
        return PathBuf::from(path);
    }

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

fn run_command(cli: &Cli, cfg: &Config, config_path: &Path) -> Result<()> {
    match &cli.command {
        Commands::Config(cmd) => commands::config::execute(cmd, cfg, config_path)
            .context("config command failed")?,
        Commands::Greet(args) => commands::greet::execute(args, cfg)
            .context("greet command failed")?,
    }
    Ok(())
}
