use clap::{Parser, Subcommand};

pub const DEFAULT_APP_NAME: &str = "max";
pub const APP_DESCRIPTION: &str = "Internal workflows and troubleshooting utility";

#[derive(Parser)]
#[command(
    name = DEFAULT_APP_NAME,
    about = APP_DESCRIPTION,
    version,
)]
pub struct Cli {
    #[arg(short, long, global = true, help = "Path to config file")]
    pub config_file: Option<String>,

    #[arg(short, long, global = true, help = "Enable verbose output")]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Manage application configuration", subcommand)]
    Config(ConfigCommands),

    #[command(about = "Print a personalized greeting message")]
    Greet(GreetArgs),
}

#[derive(clap::Args)]
pub struct GreetArgs {
    pub name: Option<String>,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    #[command(about = "Initialize a default config file")]
    Init {
        #[arg(short, long, help = "Overwrite existing config file")]
        force: bool,
    },
    #[command(about = "Display the current configuration")]
    Show {
        #[arg(short, long, help = "Output as JSON")]
        json: bool,
    },
    #[command(about = "Print the config file path")]
    Path,
}
