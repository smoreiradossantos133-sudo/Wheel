# Wheel Compiler - Multi-OS Release

This directory contains pre-compiled `wheelc` (Wheel compiler) executables for Linux, Windows, and macOS.

## Quick Start

### Linux Users
```bash
cp dist/Linux/wheelc ~/.local/bin/
chmod +x ~/.local/bin/wheelc

# First run will auto-setup PATH
wheelc examples/for_range_demo.wheel -o demo --mode ll
./demo
```

### Windows Users
```cmd
copy dist\Windows\wheelc.exe C:\Windows\System32\
# Or add dist\Windows to your system PATH

wheelc examples\for_range_demo.wheel -o demo --mode ge
demo.exe
```

### macOS Users
See `dist/MacOS/BUILD_INSTRUCTIONS.txt` to build natively on macOS.

## Installation

Each OS has a `wheelc` executable that is self-contained and relocatable. 

### Automatic Installation
```bash
# Linux/macOS
sudo ./scripts/install_wheel.sh

# This will:
# 1. Copy wheelc to /usr/local/wheel/bin/
# 2. Add /usr/local/wheel/bin to system PATH via /etc/profile.d/wheel_path.sh
```

### Manual Installation
1. Copy the appropriate executable for your OS to a directory in your PATH
2. Make it executable: `chmod +x wheelc`
3. First run will auto-configure PATH in your shell profiles

## Features

- ✅ **LLVM Backend** (Linux): Optimized native code generation via LLVM IR
- ✅ **Assembly Backend** (All OS): Direct assembly code generation
- ✅ **for in range loops**: `for i in range(10) { ... }`
- ✅ **First-run setup**: Auto-adds to PATH on first execution
- ✅ **Cross-platform**: Single build process generates all OS binaries

## Build From Source

```bash
# All targets (requires mingw-w64 for Windows)
./scripts/build_multi.sh

# Linux only
cargo build --release --features llvm

# Windows (assembly mode, no LLVM)
cargo build --release --target x86_64-pc-windows-gnu

# macOS (requires native macOS)
cargo build --release --target x86_64-apple-darwin
```

## Executable Details

| OS | Path | Format | Size | Status |
|---|---|---|---|---|
| Linux | `dist/Linux/wheelc` | ELF x86-64 | 1.3M | ✅ Ready |
| Windows | `dist/Windows/wheelc.exe` | PE x86-64 | 2.2M | ✅ Ready |
| macOS | `dist/MacOS/wheelc` | Mach-O x86-64 | Pending | ⚠️ Build on macOS |

## Compiler Modes

```bash
# LLVM mode (Linux): Optimized IR code generation
wheelc input.wheel -o output --mode ll

# Assembly mode (All OS): Direct x86 assembly code
wheelc input.wheel -o output --mode ge

# Flat binary (Linux/macOS): Raw binary without headers
wheelc input.wheel -o output.bin --mode gb
```

## First-Run Behavior

On first execution, `wheelc` will:
1. Detect your system type (Linux/Windows/macOS)
2. Identify the executable's directory
3. Add this directory to your shell's PATH in:
   - Linux/macOS: `~/.profile`, `~/.bashrc`, `~/.zshrc`
   - Windows: System PATH (requires admin)
4. Create marker `~/.wheel_installed` to skip repeat setup

You may need to reload your shell after first run:
```bash
source ~/.bashrc  # or ~/.zshrc or ~/.profile
```

## Examples

See `examples/` for sample Wheel programs:
- `for_range_demo.wheel`: Demonstrates for-in-range loops
- `calculator.wheel`: Interactive calculator
- `guessing_game.wheel`: Number guessing game
- And more...

```bash
wheelc examples/for_range_demo.wheel -o demo --mode ll
./demo
```

## Supported Language Features

- Variables: `let x = 5`
- Input: `let name = input()`
- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparisons: `<`, `>`, `<=`, `>=`, `==`, `!=`
- Conditionals: `if`, `else`
- Loops: `while`, `for in range`
- Functions: `func name() { ... }`
- Libraries: `use #memory`, `use #hwio`, etc.
- Print: `print(...)`

## Documentation

- See `QUICKSTART.md` for detailed language guide
- See `LIBRARIES.md` for standard library documentation
- See `dist/README.md` for per-OS executable information
