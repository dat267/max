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

fn execute_add(args: CmdAddArgs) -> Result<()> {
    if !Path::new("src/cli.rs").exists() {
        anyhow::bail!("no src/cli.rs found (run 'max init' first)");
    }
    if !Path::new("src/config.rs").exists() {
        anyhow::bail!("no src/config.rs found — 'cmd add' must be run from a project created by 'max init', not from the max tool itself");
    }

    let segments: Vec<&str> = args.name.split('.').collect();
    if segments.is_empty() || segments[0].is_empty() {
        anyhow::bail!("command name is required");
    }

    let desc = args.desc.unwrap_or_else(|| format!("{} command", segments.last().unwrap()));
    let leaf = segments.last().unwrap();
    let struct_name = pascal_case(leaf);
    let mod_name = leaf.to_lowercase();

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

    // Add enum variant to Commands enum - find its closing brace
    let variant = format!(
        "    #[command(about = \"{desc}\")]
    {struct_name}({struct_name}),
",
    );
    // Find the closing } of the Commands enum and insert before it
    if let Some(enum_start) = cli_content.find("pub enum Commands") {
        let rest = &cli_content[enum_start..];
        let mut depth = 0;
        let mut insert_at = None;
        for (i, c) in rest.char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        insert_at = Some(enum_start + i);
                        break;
                    }
                }
                _ => {}
            }
        }
        if let Some(pos) = insert_at {
            let (before, after) = cli_content.split_at(pos);
            let new_content = format!("{}{}{}", before.trim_end(), "\n".to_string() + variant.trim_end() + "\n", after);
            fs::write(cli_path, new_content)?;
            println!("Updated {cli_path} with {struct_name} command");
        }
    }

    // Add match arm to run_command in main.rs
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
        // Insert before the match's closing brace
        if let Some(pos) = main_content.rfind("\n    }\n    Ok(())\n}") {
            let insert_at = pos + 1;
            let (before, after) = main_content.split_at(insert_at);
            let new_content = format!("{}{}{}", before, arm, after);
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
        if name.ends_with(".rs") && name != "mod.rs" && name != "cmd.rs" && name != "init.rs" && name != "config.rs" {
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
    let path = format!("src/commands/{}.rs", args.name.to_lowercase());
    if !Path::new(&path).exists() {
        anyhow::bail!("command {:?} not found in src/commands/", args.name);
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
