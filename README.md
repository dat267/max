# max — Opinionated Rust CLI Framework

A scalable, opinionated base CLI application in Rust — the template I use for every new CLI I build. Modeled after a Go equivalent with `kong`, ported to idiomatic Rust with `clap`.

## Install

**Binary** — download from [GitHub Releases](https://github.com/dat267/max/releases):

**Linux (x86_64):**
```bash
curl -sSfL https://github.com/dat267/max/releases/latest/download/max-x86_64-unknown-linux-gnu -o ~/.local/bin/max
chmod +x ~/.local/bin/max
```

**macOS (arm64):**
```bash
curl -sSfL https://github.com/dat267/max/releases/latest/download/max-aarch64-apple-darwin -o ~/.local/bin/max
chmod +x ~/.local/bin/max
```

**Windows (x86_64):**
```powershell
mkdir -Force ~\.local\bin >$null
curl -sSfL https://github.com/dat267/max/releases/latest/download/max-x86_64-pc-windows-msvc.exe -o ~\.local\bin\max.exe
```

**From source**:

```bash
cargo install --git https://github.com/dat267/max
```

## Philosophy

Every CLI I write needs the same boilerplate: config file resolution, environment variable overrides, subcommand dispatch, and layered configuration merging. This template bakes all of that in so each new project starts from a solid foundation rather than `fn main()`.

## Quick Start

**Use as a template:**

Click **"Use this template"** at the top of the [GitHub page](https://github.com/dat267/max), then clone your new repo:

```bash
git clone git@github.com:your-org/my-cli.git
cd my-cli
```

**Clean up template artifacts:**

```bash
# Remove existing tags and release history
git tag | xargs git tag -d
git remote remove origin

# Rename everywhere (replace "max" with your app name):
#   - Cargo.toml        → package.name
#   - src/main.rs       → DEFAULT_APP_NAME
#   - src/cli.rs        → DEFAULT_APP_NAME
#   - README.md         → title, install URLs
#   - .github/workflows → BIN_NAME (via repo variable)
```

Then push to your own repository.

**Build and run:**

```bash
cargo build
cargo run -- greet
cargo run -- config init
cargo run -- config show
```

## Project Structure

```
src/
  main.rs              # Entry point: app name resolution, config loading, dispatch
  cli.rs               # clap CLI definitions (Cli, Commands, subcommand enums)
  config.rs            # Config struct, JSON loading, env override, ConfigDefaults trait
  commands/
    mod.rs             # Module exports
    config.rs          # config {init, show, path} subcommand
    greet.rs           # greet subcommand — example to copy when adding commands
```

## Configuration

### Config File Resolution

The config file is located in this priority order:

1. `--config-file PATH` flag
2. `{APP}_CONFIG_FILE` environment variable
3. `{app}.json` in the current directory
4. `~/.config/{app}/{app}.json` (XDG config directory)
5. `{app}.json` fallback

### File Format (JSON)

```json
{
  "admin-token": null,
  "core": {
    "timeout": "2m",
    "retries": 3
  },
  "debug": false,
  "dry-run": false
}
```

Field naming is `kebab-case` in JSON, which maps to `snake_case` in Rust.

### Layered Configuration — Specificity Precedence

Values are resolved with strict override specificity:

```
CLI flags  >  Environment variables  >  Config file  >  Struct defaults
```

Each layer overrides the one before it. A value provided via CLI flag takes
ultimate precedence over everything.

### Environment Variable Overrides

Every leaf config field can be overridden with `{APP}_{FLAT_KEY}`:

```bash
# Single fields
MAX_DEBUG=true my-app greet
MAX_DRY_RUN=true my-app greet

# Nested fields (kebab → underscores)
MAX_CORE_TIMEOUT=30s my-app greet
MAX_CORE_RETRIES=10 my-app greet

# Combined
MAX_DEBUG=true MAX_DRY_RUN=true MAX_CORE_TIMEOUT=5m my-app greet
```

Env-var values are typed automatically: `"true"/"1"/"yes"` → bool, `"42"` → integer, otherwise string.

### Auto-Wiring: Config Values as CLI Flag Defaults

When you add a CLI flag to any subcommand, if the flag's name (in kebab-case)
matches a key in the config file, the config value automatically becomes the
flag's default. **No manual wiring required.**

This works through the `ConfigDefaults` trait: before a command runs,
`flat_config_strings(config)` builds a flat key→value map from the config,
and `set_in_args` injects any matching value into the serialized args struct
where the field is currently `null`.

```rust
#[derive(clap::Args, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FetchArgs {
    #[arg(long)]
    pub core_timeout: Option<String>,  // auto-defaults from config.core.timeout

    #[arg(long)]
    pub retries: Option<i32>,          // auto-defaults from config.core.retries

    pub url: String,                   // no config match — local only
}
```

#### Field Naming Rules

| Rust field | CLI flag | Config key | Auto-wired? |
|---|---|---|---|
| `core_timeout` | `--core-timeout` | `core.timeout` | Yes — matches via flat key `core-timeout` |
| `dry_run` | `--dry-run` | `dry-run` | Yes — atomic kebab key |
| `admin_token` | `--admin-token` | `admin-token` | Yes |
| `url` | `URL` | _(none)_ | No — no matching config key |

If the field name (after kebab-case conversion) matches a config leaf path,
the config value becomes the CLI default. Fields with no matching config key
are purely local to the subcommand.

### Global Flags

| Flag | Description |
|------|-------------|
| `-c`, `--config-file` | Path to config file |
| `-v`, `--verbose` | Enable verbose output |
| `-h`, `--help` | Print help |
| `-V`, `--version` | Print version |

### Built-in Subcommands

#### `config`

Manage the application configuration file.

| Command | Description |
|---------|-------------|
| `config init` | Create a default config file |
| `config init --force` | Overwrite existing config |
| `config show` | Display current configuration |
| `config show --json` | Output as JSON (same as default) |
| `config path` | Print config file path |

#### `greet [name]`

Print a personalized greeting. Supports `--admin-token` which auto-defaults
from the config, falls back to `"World"`.

```bash
my-app greet                        # Hello, World!
my-app greet Alice                  # Hello, Alice!
my-app greet --admin-token bot      # Hello, bot!
MAX_ADMIN_TOKEN=bot my-app greet    # Hello, bot!
MAX_ADMIN_TOKEN=env my-app greet --admin-token cli  # Hello, cli!  (CLI wins)
```

## Adding a New Subcommand

Adding a subcommand involves three steps:

### 1. Define the Args Struct

In `src/cli.rs`, add your args struct. Derive `clap::Args`, `serde::Serialize`,
and `serde::Deserialize` with `rename_all = "kebab-case"`:

```rust
#[derive(clap::Args, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FetchArgs {
    #[arg(long, help = "Request timeout")]
    pub core_timeout: Option<String>,

    #[arg(long, help = "Max retries")]
    pub retries: Option<i32>,

    pub url: String,
}
```

Fields named to match config keys (after kebab-case conversion) will
auto-inherit their defaults from the config file or environment variables.

### 2. Register the Subcommand

Add it to the `Commands` enum:

```rust
pub enum Commands {
    Config(ConfigCommands),
    Greet(GreetArgs),
    #[command(about = "Fetch a resource")]
    Fetch(FetchArgs),
}
```

### 3. Create the Command Handler

`src/commands/fetch.rs`:

```rust
use crate::cli::FetchArgs;
use crate::config::Config;
use anyhow::Result;

pub fn execute(args: &FetchArgs, config: &Config) -> Result<()> {
    // args.core_timeout already defaults from config.core.timeout
    // args.retries already defaults from config.core.retries
    // config.debug, config.dry_run are available for global behavior
    Ok(())
}
```

### 4. Register the Module

`src/commands/mod.rs`:

```rust
pub mod config;
pub mod greet;
pub mod fetch;
```

### 5. Wire It Up

In `src/main.rs`:

```rust
Commands::Fetch(mut args) => {
    args.apply_config_defaults(cfg);
    commands::fetch::execute(&args, cfg)
        .context("fetch command failed")?
}
```

That's it — config, env vars, and CLI flags are all automatically resolved.

## Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing (derive API) |
| `serde` / `serde_json` | Configuration de/serialization |
| `dirs` | XDG config directory resolution |
| `anyhow` | Error handling with context |

## License

MIT
