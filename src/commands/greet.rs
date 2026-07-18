use crate::cli::GreetArgs;
use crate::config::Config;
use anyhow::Result;

pub fn execute(args: &GreetArgs, config: &Config) -> Result<()> {
    let name = args
        .name
        .as_deref()
        .or(config.admin_token.as_deref())
        .unwrap_or("World");

    let greeting = if config.debug {
        format!("[debug] Hello, {}!", name)
    } else {
        format!("Hello, {}!", name)
    };

    if config.dry_run {
        println!("[dry-run] Would greet: {}", name);
    } else {
        println!("{}", greeting);
    }

    Ok(())
}
