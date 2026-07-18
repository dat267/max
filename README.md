# max — Opinionated Rust CLI Framework

A scalable, opinionated base CLI application in Rust — the template I use for every new CLI I build. Modeled after a Go equivalent with `kong`, ported to idiomatic Rust with `clap`.

## Philosophy

Every CLI I write needs the same boilerplate: config file resolution, environment variable overrides, subcommand dispatch, and layered configuration merging. This template bakes all of that in so each new project starts from a solid foundation rather than `fn main()`.

## Quick Start

```bash
# Clone as a new project
git clone <this-repo> my-new-cli
cd my-new-cli

# Rename (replace all occurrences of "max" with your app name)
# Then build
cargo build

# Try it
cargo run -- greet
cargo run -- config init
cargo run -- config show
```

## Project Structure

```
src/
  main.rs              # Entry point: app name resolution, config loading, dispatch
  cli.rs               # clap CLI definitions (Cli, Commands, subcommand enums)
  config.rs            # Config struct, JSON loading, env override merging
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

### Layered Configuration

Values are merged with the following precedence (highest wins):

```
CLI flags  >  Environment variables  >  Config file  >  Struct defaults
```

Each layer overrides the one before it.

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

Print a personalized greeting. Falls back to `admin-token` from config, then `"World"`.

```bash
my-app greet              # Hello, World!
my-app greet Alice        # Hello, Alice!
MAX_ADMIN_TOKEN=bot my-app greet  # Hello, bot!
```

## Adding a New Subcommand

1. **Define CLI args** in `src/cli.rs`:
   ```rust
   #[derive(clap::Args)]
   pub struct FooArgs {
       pub bar: String,
       #[arg(short, long)]
       pub count: Option<i32>,
   }
   ```

2. **Add to the enum** in `src/cli.rs`:
   ```rust
   pub enum Commands {
       // ... existing commands ...
       #[command(about = "Do foo things")]
       Foo(FooArgs),
   }
   ```

3. **Create `src/commands/foo.rs`**:
   ```rust
   use crate::cli::FooArgs;
   use crate::config::Config;
   use anyhow::Result;

   pub fn execute(args: &FooArgs, config: &Config) -> Result<()> {
       // Your logic here — config is fully merged at this point
       Ok(())
   }
   ```

4. **Register it** in `src/commands/mod.rs`:
   ```rust
   pub mod config;
   pub mod greet;
   pub mod foo;
   ```

5. **Wire it up** in `src/main.rs`:
   ```rust
   Commands::Foo(args) => commands::foo::execute(args, cfg)
       .context("foo command failed")?,
   ```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing (derive API) |
| `serde` / `serde_json` | Configuration de/serialization |
| `dirs` | XDG config directory resolution |
| `anyhow` | Error handling with context |

## License

MIT
