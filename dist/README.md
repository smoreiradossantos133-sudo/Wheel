# Wheel Compiler Executables

This directory contains pre-built `wheelc` (Wheel compiler) executables for different operating systems.

## Available Executables

### Linux (`Linux/wheelc`)
- **Format**: ELF (x86_64)
- **Size**: ~1.3M
- **Status**: ✅ Ready to use
- **Usage**: 
  ```bash
  ./Linux/wheelc examples/for_range_demo.wheel -o demo --mode ll
  ./demo
  ```

### Windows (`Windows/wheelc.exe`)
- **Format**: PE/EXE (x86_64)
- **Size**: ~2.2M
- **Status**: ✅ Ready to use (requires Windows or Wine)
- **Usage**:
  ```cmd
  Windows\wheelc.exe examples\for_range_demo.wheel -o demo --mode ll
  demo.exe
  ```

### macOS (`MacOS/wheelc`)
- **Format**: Mach-O (x86_64)
- **Status**: ⚠️ To build, compile natively on macOS
- **Instructions**:
  1. Clone this repository on macOS
  2. Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
  3. Build: `cargo build --release`
  4. Binary will be at `target/release/wheelc`
  5. Copy to this folder: `cp target/release/wheelc dist/MacOS/wheelc`

## Installation

### Linux/WSL
```bash
# Copy to user local bin
cp Linux/wheelc ~/.local/bin/wheelc
chmod +x ~/.local/bin/wheelc

# Or system-wide (requires sudo)
sudo cp Linux/wheelc /usr/local/bin/wheelc
sudo chmod +x /usr/local/bin/wheelc
```

### Windows
```cmd
REM Copy to system path (requires Administrator)
copy Windows\wheelc.exe C:\Windows\System32\wheelc.exe

REM Or add to user PATH
```

### macOS
```bash
# Copy to user local bin
cp MacOS/wheelc ~/.local/bin/wheelc
chmod +x ~/.local/bin/wheelc

# Or system-wide
sudo cp MacOS/wheelc /usr/local/bin/wheelc
sudo chmod +x /usr/local/bin/wheelc
```

## First-Run Setup

When you run `wheelc` for the first time, it will:
1. Detect your system's root directory
2. Add the executable's directory to your shell PATH
3. Create a marker file `~/.wheel_installed` to avoid repeating

After the first run, you may need to reload your shell profile:
```bash
source ~/.profile  # or ~/.bashrc or ~/.zshrc
```

## Building from Source

To build all executables:
```bash
# Install dependencies
rustup target add x86_64-unknown-linux-gnu x86_64-pc-windows-gnu

# Build for all targets
./scripts/build_multi.sh
```

## Features

- ✅ `for in range` loops: `for i in range(10) { print(i); }`
- ✅ LLVM backend (Linux): `-mode ll` for optimized native code
- ✅ Assembly backend (all platforms): `-mode ge` for generated executables
- ✅ First-run PATH installation
- ✅ Cross-platform compilation support

## License

See LICENSE file in the repository root.
