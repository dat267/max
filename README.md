# MaxCLI

A production-grade, extensible CLI application built with Go.

## Structure

- `cmd/`: Application entry points.
- `internal/`: Private application code (not importable by others).
- `pkg/`: Public library code that can be imported.
- `configs/`: Default configuration files.
- `scripts/`: Development and automation scripts.

## Installation

### From Source
```bash
go install github.com/dat267/max/cmd/max@latest
```

### Binaries
Download pre-built binaries from [Releases](https://github.com/dat267/max/releases).

**Linux/macOS (curl):**
```bash
# Download and rename to 'max'
curl -Lo max https://github.com/dat267/max/releases/latest/download/max-linux-amd64
chmod +x max
```

**Linux/macOS (wget):**
```bash
# Download and rename to 'max'
wget -O max https://github.com/dat267/max/releases/latest/download/max-linux-amd64
chmod +x max
```

**Windows (PowerShell):**
```powershell
# Download and save as 'max.exe'
irm https://github.com/dat267/max/releases/latest/download/max-windows-amd64.exe -OutFile max.exe
```
