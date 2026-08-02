use crate::cli::{CmdAddArgs, CmdArgs, CmdCommands, CmdEditArgs};
use crate::config::Config;
use anyhow::Result;
use std::fs;
use std::path::Path;

const ENUM_MARKER: &str = "// __CMD_ENUM_MARKER__";
const DISPATCH_MARKER: &str = "// __CMD_DISPATCH_MARKER__";

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

fn derive_command(name: &str) -> Result<(String, String, String)> {
    validate_name(name)?;
    let segments: Vec<&str> = name.split('.').collect();
    if segments.iter().any(|s| s.is_empty()) {
        anyhow::bail!("command name contains empty segments (e.g. consecutive dots)");
    }
    let leaf = segments.last().unwrap();
    if !is_valid_rust_ident(leaf) {
        anyhow::bail!("command name {leaf:?} is not a valid Rust identifier");
    }
    Ok((leaf.to_string(), pascal_case(leaf), leaf.to_lowercase()))
}

fn execute_add(args: CmdAddArgs) -> Result<()> {
    if !Path::new("src/cli.rs").exists() {
        anyhow::bail!("no src/cli.rs found (run 'max init' first)");
    }
    if !Path::new("src/config.rs").exists() {
        anyhow::bail!("no src/config.rs found — run 'max init' first or check you are in a project directory");
    }

    let (leaf, struct_name, mod_name) = derive_command(&args.name)?;

    // Create command handler file
    let handler_path = format!("src/commands/{mod_name}.rs");
    if Path::new(&handler_path).exists() {
        anyhow::bail!("command {leaf:?} already exists");
    }
    fs::write(&handler_path, build_handler_source(&struct_name))?;
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
    let new_content = format!("{}{}", cli_content.trim_end(), build_struct_def(&struct_name));
    fs::write(cli_path, new_content)?;

    // Re-read file after struct insertion
    let cli_content = fs::read_to_string(cli_path)?;

    // Add enum variant to Commands enum via sentinel marker
    let desc = args.desc.unwrap_or_else(|| format!("{} command", leaf));
    let new_cli_content =
        insert_enum_variant(&cli_content, &build_enum_variant(&struct_name, &desc))?;
    fs::write(cli_path, new_cli_content)?;
    println!("Updated {cli_path} with {struct_name} command");

    // Add match arm to run_command via sentinel marker
    let main_path = "src/main.rs";
    if Path::new(main_path).exists() {
        let main_content = fs::read_to_string(main_path)?;
        let arm = build_dispatch_arm(&mod_name, &struct_name);
        let new_main_content = insert_dispatch_arm(&main_content, &arm)?;
        fs::write(main_path, new_main_content)?;
        println!("Updated {main_path}");
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
    let (leaf, _, mod_name) = derive_command(&args.name)?;

    let path = format!("src/commands/{mod_name}.rs");
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

fn build_handler_source(struct_name: &str) -> String {
    format!(
        "use crate::cli::{struct_name};
use crate::config::Config;
use anyhow::Result;
use std::path::Path;

pub fn execute(_args: &{struct_name}, _cfg: &Config, _config_path: &Path) -> Result<()> {{
    println!(\"TODO: implement {struct_name} command\");
    Ok(())
}}
",
    )
}

fn build_struct_def(struct_name: &str) -> String {
    format!(
        "
#[derive(clap::Args)]
pub struct {struct_name} {{
    #[arg(long, help = \"Name\")]
    pub name: Option<String>,
}}
",
    )
}

fn build_enum_variant(struct_name: &str, desc: &str) -> String {
    format!(
        "    #[command(about = \"{desc}\")]
    {struct_name}({struct_name}),
",
    )
}

fn build_dispatch_arm(mod_name: &str, struct_name: &str) -> String {
    format!(
        "        Commands::{struct_name}(args) => {{
            commands::{mod_name}::execute(&args, cfg, config_path)
                .context(\"{mod_name} command failed\")?
        }}
",
    )
}

fn insert_before_marker(content: &str, marker: &str, insertion: &str) -> Result<String> {
    let pos = content
        .find(marker)
        .ok_or_else(|| anyhow::anyhow!("codegen marker {marker:?} not found in generated project files"))?;
    let (before, after) = content.split_at(pos);
    Ok(format!("{}{}\n{}", before, insertion.trim_end(), after.trim_start()))
}

fn insert_enum_variant(cli_content: &str, variant_source: &str) -> Result<String> {
    insert_before_marker(cli_content, ENUM_MARKER, variant_source)
}

fn insert_dispatch_arm(main_content: &str, arm_source: &str) -> Result<String> {
    insert_before_marker(main_content, DISPATCH_MARKER, arm_source)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pascal_case_converts_words() {
        assert_eq!(pascal_case("foo"), "Foo");
        assert_eq!(pascal_case("foo-bar"), "FooBar");
        assert_eq!(pascal_case("foo.bar"), "FooBar");
        assert_eq!(pascal_case("foo_bar"), "FooBar");
        assert_eq!(pascal_case(""), "");
    }

    #[test]
    fn is_valid_rust_ident_accepts_valid() {
        assert!(is_valid_rust_ident("Foo"));
        assert!(is_valid_rust_ident("_x"));
        assert!(is_valid_rust_ident("Foo2"));
    }

    #[test]
    fn is_valid_rust_ident_rejects_invalid() {
        assert!(!is_valid_rust_ident("2Foo"));
        assert!(!is_valid_rust_ident("foo-bar"));
        assert!(!is_valid_rust_ident(""));
    }

    #[test]
    fn validate_name_rejects_unsafe() {
        assert!(validate_name("").is_err());
        assert!(validate_name("../x").is_err());
        assert!(validate_name("a/b").is_err());
        assert!(validate_name("a\\b").is_err());
        assert!(validate_name("a\0b").is_err());
        assert!(validate_name("ok.name").is_ok());
    }

    #[test]
    fn derive_command_uses_leaf_segment() {
        let (leaf, struct_name, mod_name) = derive_command("admin.users.list").unwrap();
        assert_eq!(leaf, "list");
        assert_eq!(struct_name, "List");
        assert_eq!(mod_name, "list");
    }

    #[test]
    fn derive_command_rejects_invalid_leaf() {
        assert!(derive_command("foo-bar").is_err());
        assert!(derive_command("2bad").is_err());
        assert!(derive_command("a..b").is_err());
        assert!(derive_command("").is_err());
    }

    #[test]
    fn handler_source_uses_underscored_args() {
        let src = build_handler_source("FooCmd");
        assert!(src.contains("pub fn execute(_args: &FooCmd"));
        assert!(src.contains("TODO: implement FooCmd command"));
    }

    #[test]
    fn struct_def_contains_struct_name() {
        let src = build_struct_def("FooCmd");
        assert!(src.contains("pub struct FooCmd"));
        assert!(src.contains("pub name: Option<String>,"));
    }

    #[test]
    fn enum_variant_inserted_before_marker() {
        let content = "enum Commands {\n    Init(InitArgs),\n    // __CMD_ENUM_MARKER__\n}\n";
        let variant = build_enum_variant("FooCmd", "foo command");
        let out = insert_enum_variant(content, &variant).unwrap();
        assert!(out.contains("FooCmd(FooCmd),"));
        assert!(out.contains("// __CMD_ENUM_MARKER__"));
        let marker_pos = out.find("// __CMD_ENUM_MARKER__").unwrap();
        let variant_pos = out.find("FooCmd(FooCmd),").unwrap();
        assert!(variant_pos < marker_pos);
    }

    #[test]
    fn dispatch_arm_inserted_before_marker() {
        let content = "match cli.command {\n    // __CMD_DISPATCH_MARKER__\n}\n";
        let arm = build_dispatch_arm("foo", "FooCmd");
        let out = insert_dispatch_arm(content, &arm).unwrap();
        assert!(out.contains("Commands::FooCmd(args)"));
        assert!(out.contains("commands::foo::execute"));
        assert!(out.contains("// __CMD_DISPATCH_MARKER__"));
    }

    #[test]
    fn enum_insert_errors_when_marker_missing() {
        let content = "enum Commands {\n    Init(InitArgs),\n}\n";
        let variant = build_enum_variant("FooCmd", "foo command");
        assert!(insert_enum_variant(content, &variant).is_err());
    }

    #[test]
    fn dispatch_insert_errors_when_marker_missing() {
        let content = "match cli.command {\n}\n";
        let arm = build_dispatch_arm("foo", "FooCmd");
        assert!(insert_dispatch_arm(content, &arm).is_err());
    }
}
