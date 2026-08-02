use crate::cli::InitArgs;
use crate::config::Config;
use anyhow::Result;
use std::fs;
use std::path::Path;

struct Template {
    path: &'static str,
    content: &'static str,
}

const TEMPLATES: &[Template] = &[
    Template { path: "Cargo.toml", content: include_str!("../../templates/Cargo.toml") },
    Template { path: ".gitignore", content: include_str!("../../templates/.gitignore") },
    Template { path: ".github/workflows/release.yml", content: include_str!("../../templates/.github/workflows/release.yml") },
    Template { path: "src/main.rs", content: include_str!("../../templates/src/main.rs") },
    Template { path: "src/cli.rs", content: include_str!("../../templates/src/cli.rs") },
    Template { path: "src/config.rs", content: include_str!("../../templates/src/config.rs") },
    Template { path: "src/commands/mod.rs", content: include_str!("../../templates/src/commands/mod.rs") },
    Template { path: "src/commands/greet.rs", content: include_str!("../../templates/src/commands/greet.rs") },
    Template { path: "src/commands/config.rs", content: include_str!("../../templates/src/commands/config.rs") },
];

pub fn execute(args: InitArgs, _cfg: &Config, _config_path: &Path) -> Result<()> {
    let dir = &args.name;

    if dir.is_empty() {
        anyhow::bail!("project name is required");
    }
    if dir.contains('/') || dir.contains('\\') || dir.contains("..") || dir.contains('\0') {
        anyhow::bail!("invalid project name: {dir:?}");
    }

    let path = Path::new(dir);
    if path.exists() {
        anyhow::bail!("directory {dir:?} already exists");
    }

    for tmpl in TEMPLATES {
        let dest_path = path.join(tmpl.path);
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = tmpl.content.replace("{{project_name}}", dir);
        fs::write(&dest_path, content.as_bytes())?;
    }

    println!("Created project {dir:?} in {dir}/");
    println!("  cd {dir} && cargo build");
    Ok(())
}
