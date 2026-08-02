# max — CLI project scaffolding tool

Generate new CLI projects, add commands, manage config.

```
Usage: max <command>

CLI project scaffolding tool

Commands:
  init           Initialize a new CLI project
  cmd add        Add a new command
  cmd show       List all commands
  cmd edit       Edit a command struct
  config init    Generate a default configuration file
  config set     Set a config value
  config unset   Unset a config value
  config path    Show configuration file path
  config show    Print current configuration values
  config edit    Edit configuration file

Flags:
  -h, --help    Show context-sensitive help.
  -V, --version Print version
```

## Quick start

```bash
# Install
cargo install --git https://github.com/dat267/max

# Create a project — includes greet + config commands out of the box
max init mycli
cd mycli && cargo run -- greet
# → Hello, World!

# Add a command
max cmd add hello
cargo run -- hello
# → TODO: implement Hello command

# Config commands come built-in
cargo run -- config init
cargo run -- config show
```

## Generated project structure

```
mycli/
  Cargo.toml
  src/
    main.rs       — entry point, clap dispatch
    cli.rs        — CLI struct + config commands
    config.rs     — Config struct, env override, config_defaults! macro
    commands/
      mod.rs
      greet.rs    — example Greet command
      config.rs   — config {init, show, path} subcommands
```

A generated project includes:
- **Greet** example command with `--admin-token` flag
- **Config** commands (`init`, `show`, `path`)
- `--verbose` root-level flag
- Config file resolution (`$<APP>_CONFIG_FILE` env > local `<app>.json` > XDG config dir)
- Env var overrides (`APP_KEY=value`)
- `config_defaults!` macro — type-safe auto-wiring of config values into CLI args

## Adding commands

```bash
max cmd add <name>        # add a command
max cmd add --desc "..."  # with description
max cmd add admin.users   # flat command named after the leaf segment (users)
```

Command names may use dot-separated paths, but `max` only uses the last
segment: `max cmd add admin.users` generates a single flat `Users` command,
not nested subcommands. Every segment must form a valid Rust identifier.

## Config set

```bash
# Flat key
max config set greeting hello
# → { "greeting": "hello" }

# Nested key (dot notation)
max config set core.timeout 5m
# → { "core": { "timeout": "5m" } }

# Custom config file
max config set --config-file /path/to/config.json key value
```

## Development

```bash
cargo build
cargo run -- init test-cli
cargo run -- cmd add foo
cargo run -- config set foo bar
```

## License

MIT
