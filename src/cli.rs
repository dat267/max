use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "max",
    about = "CLI project scaffolding tool",
    version,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Initialize a new CLI project")]
    Init(InitArgs),
    #[command(about = "Manage commands in a project")]
    Cmd(CmdArgs),
    #[command(about = "Manage application configuration", subcommand)]
    Config(ConfigCommands),
}

#[derive(clap::Args)]
pub struct InitArgs {
    pub name: String,
}

#[derive(clap::Args)]
pub struct CmdArgs {
    #[command(subcommand)]
    pub command: CmdCommands,
}

#[derive(Subcommand)]
pub enum CmdCommands {
    #[command(about = "Add a new command")]
    Add(CmdAddArgs),
    #[command(about = "List all commands")]
    Show,
    #[command(about = "Edit a command struct")]
    Edit(CmdEditArgs),
}

#[derive(clap::Args)]
pub struct CmdAddArgs {
    pub name: String,
    #[arg(long, help = "Description for the command")]
    pub desc: Option<String>,
}

#[derive(clap::Args)]
pub struct CmdEditArgs {
    pub name: String,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    #[command(about = "Generate a default configuration file")]
    Init(ConfigInitArgs),
    #[command(about = "Set a config value")]
    Set(ConfigSetArgs),
    #[command(about = "Unset a config value")]
    Unset(ConfigUnsetArgs),
    #[command(about = "Show configuration file path")]
    Path,
    #[command(about = "Print current configuration values")]
    Show,
    #[command(about = "Edit configuration file")]
    Edit,
}

#[derive(clap::Args)]
pub struct ConfigInitArgs {
    #[arg(short, long, help = "Overwrite existing file")]
    pub force: bool,
}

#[derive(clap::Args)]
pub struct ConfigSetArgs {
    #[arg(short, long, help = "Config file path")]
    pub config_file: Option<String>,
    pub key: String,
    pub value: String,
}

#[derive(clap::Args)]
pub struct ConfigUnsetArgs {
    #[arg(short, long, help = "Config file path")]
    pub config_file: Option<String>,
    pub key: String,
}
