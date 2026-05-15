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

**Linux/macOS (curl - AMD64):**
```bash
# Download and rename to 'max'
curl -Lo max https://github.com/dat267/max/releases/latest/download/max-linux-amd64
chmod +x max
```

**Linux/macOS (curl - ARM64):**
```bash
# Download and rename to 'max'
curl -Lo max https://github.com/dat267/max/releases/latest/download/max-linux-arm64
chmod +x max
```

**Linux/macOS (wget - AMD64):**
```bash
# Download and rename to 'max'
wget -O max https://github.com/dat267/max/releases/latest/download/max-linux-amd64
chmod +x max
```

**Linux/macOS (wget - ARM64):**
```bash
# Download and rename to 'max'
wget -O max https://github.com/dat267/max/releases/latest/download/max-linux-arm64
chmod +x max
```

**Windows (PowerShell - AMD64):**
```powershell
# Download and save as 'max.exe'
irm https://github.com/dat267/max/releases/latest/download/max-windows-amd64.exe -OutFile max.exe
```

**Windows (PowerShell - ARM64):**
```powershell
# Download and save as 'max.exe'
irm https://github.com/dat267/max/releases/latest/download/max-windows-arm64.exe -OutFile max.exe
```
