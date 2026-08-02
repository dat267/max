mod cli;
mod commands;
mod config;

use anyhow::{Context, Result};
use clap::Parser;
use std::path::Path;

use cli::{Cli, Commands};
use config::Config;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_path = config::config_path();
    let cfg = config::load_config();
    run_command(cli, &cfg, &config_path)
}

fn run_command(cli: Cli, cfg: &Config, config_path: &Path) -> Result<()> {
    match cli.command {
        Commands::Init(args) => commands::init::execute(args, cfg, config_path)
            .context("init command failed")?,
        Commands::Cmd(args) => commands::cmd::execute(args, cfg, config_path)
            .context("cmd command failed")?,
        Commands::Config(cmd) => commands::config::execute(cmd, cfg, config_path)
            .context("config command failed")?,
        // __CMD_DISPATCH_MARKER__
    }
    Ok(())
}
