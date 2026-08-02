use crate::cli::{CmdAddArgs, CmdArgs, CmdCommands, CmdEditArgs};
use crate::config::Config;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn execute(args: CmdArgs, _cfg: &Config, _config_path: &Path) -> Result<()> {
    match args.command {
        CmdCommands::Add(add_args) => execute_add(add_args),
        CmdCommands::Show => execute_show(),
        CmdCommands::Edit(edit_args) => execute_edit(edit_args),
    }
}

fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        anyhow::bail!("command name is required");
    }
    if name.contains("..") || name.contains('/') || name.contains('\\') || name.contains('\0') {
        anyhow::bail!("invalid command name: {name:?}");
    }
    Ok(())
}

fn execute_add(args: CmdAddArgs) -> Result<()> {
    if !Path::new("src/cli.rs").exists() {
        anyhow::bail!("no src/cli.rs found (run 'max init' first)");
    }
    if !Path::new("src/config.rs").exists() {
        anyhow::bail!("no src/config.rs found — run 'max init' first or check you are in a project directory");
    }

    validate_name(&args.name)?;

    let segments: Vec<&str> = args.name.split('.').collect();
    if segments.iter().any(|s| s.is_empty()) {
        anyhow::bail!("command name contains empty segments (e.g. consecutive dots)");
    }

    let leaf = segments.last().unwrap();
    let struct_name = pascal_case(leaf);
    let mod_name = leaf.to_lowercase();

    if !is_valid_rust_ident(&struct_name) || struct_name.is_empty() {
        anyhow::bail!("command name {leaf:?} does not produce a valid Rust identifier");
    }

    // Create command handler file
    let handler_path = format!("src/commands/{mod_name}.rs");
    if Path::new(&handler_path).exists() {
        anyhow::bail!("command {leaf:?} already exists");
    }
    let handler = format!(
        "use crate::cli::{struct_name};
use crate::config::Config;
use anyhow::Result;
use std::path::Path;

pub fn execute(args: &{struct_name}, _cfg: &Config, _config_path: &Path) -> Result<()> {{
    println!(\"TODO: implement {struct_name} command\");
    Ok(())
}}
",
    );
    fs::write(&handler_path, handler)?;
    println!("Created {handler_path}");

    // Register module in commands/mod.rs
    let mod_path = "src/commands/mod.rs";
    let mod_content = fs::read_to_string(mod_path)?;
    let new_mod_content = format!("{}\npub mod {mod_name};\n", mod_content.trim_end());
    fs::write(mod_path, new_mod_content)?;
    println!("Updated {mod_path}");

    // Add args struct to cli.rs
    let cli_path = "src/cli.rs";
    let cli_content = fs::read_to_string(cli_path)?;
    let struct_def = format!(
        "\n#[derive(clap::Args)]
pub struct {struct_name} {{
    #[arg(long, help = \"Name\")]
    pub name: Option<String>,
}}
",
    );

    // Append struct definition at end of file
    let new_content = format!("{}{}", cli_content.trim_end(), struct_def);
    fs::write(cli_path, new_content)?;

    // Re-read file after struct insertion
    let cli_content = fs::read_to_string(cli_path)?;

    // Add enum variant to Commands enum via sentinel marker
    let desc = args.desc.unwrap_or_else(|| format!("{} command", leaf));
    let variant = format!(
        "    #[command(about = \"{desc}\")]
    {struct_name}({struct_name}),
",
    );
    if let Some(pos) = cli_content.find("// __CMD_ENUM_MARKER__") {
        let (before, after) = cli_content.split_at(pos);
        let new_content = format!("{}{}{}", before, variant.trim_end().to_owned() + "\n", after.trim_start());
        fs::write(cli_path, new_content)?;
        println!("Updated {cli_path} with {struct_name} command");
    }

    // Add match arm to run_command via sentinel marker
    let main_path = "src/main.rs";
    if Path::new(main_path).exists() {
        let main_content = fs::read_to_string(main_path)?;

        let arm = format!(
            "        Commands::{struct_name}(args) => {{
            commands::{mod_name}::execute(&args, cfg, config_path)
                .context(\"{mod_name} command failed\")?
        }}
",
        );
        if let Some(pos) = main_content.find("// __CMD_DISPATCH_MARKER__") {
            let (before, after) = main_content.split_at(pos);
            let new_content = format!("{}{}{}", before, arm.trim_end().to_owned() + "\n", after.trim_start());
            fs::write(main_path, new_content)?;
            println!("Updated {main_path}");
        }
    }

    println!("Added command {leaf:?}");
    Ok(())
}

fn execute_show() -> Result<()> {
    let entries = fs::read_dir("src/commands")?;
    println!("Commands:");
    let mut count = 0;
    for entry in entries {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".rs") && name != "mod.rs" && name != "greet.rs" && name != "config.rs" {
            println!("  {}", name.trim_end_matches(".rs"));
            count += 1;
        }
    }
    if count == 0 {
        println!("  (no commands yet)");
    }
    Ok(())
}

fn execute_edit(args: CmdEditArgs) -> Result<()> {
    validate_name(&args.name)?;

    let segments: Vec<&str> = args.name.split('.').collect();
    let leaf = segments.last().unwrap();
    if leaf.is_empty() {
        anyhow::bail!("invalid command name");
    }

    let path = format!("src/commands/{}.rs", leaf.to_lowercase());
    if !Path::new(&path).exists() {
        anyhow::bail!("command {leaf:?} not found in src/commands/");
    }
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    let status = std::process::Command::new(&editor).arg(&path).status()?;
    if !status.success() {
        anyhow::bail!("editor exited with error");
    }
    Ok(())
}

fn pascal_case(s: &str) -> String {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect()
}

fn is_valid_rust_ident(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let first = s.chars().next().unwrap();
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }
    s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}
