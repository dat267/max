use clap::{Parser, Subcommand};
use crate::config_defaults;

pub const DEFAULT_APP_NAME: &str = "{{project_name}}";

#[derive(Parser)]
#[command(
    name = DEFAULT_APP_NAME,
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

    #[command(about = "Print a personalized greeting")]
    Greet(GreetArgs),
}

#[derive(clap::Args)]
pub struct GreetArgs {
    #[arg(long, help = "Admin token for authentication")]
    pub admin_token: Option<String>,

    pub name: Option<String>,
}

config_defaults!(GreetArgs {
    admin_token => (admin_token),
});

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
