# Wheel (MVP compiler)

This repository contains a minimal prototype of the `wheel` compiler (here named `wheelc`) written in Rust. The goal is to create a medium-level programming language called *Wheel* that will eventually generate native binaries and executables directly.

MVP features implemented here:
- CLI: `wheelc <input.wheel> -o <output> --mode ge|gb|ll`
  - `ge` -> generates native executable (ELF x86_64 on Linux) 
  - `gb` -> generates flat binary (raw binary) by producing an executable then `objcopy`
  - `ll` -> generates executable via LLVM backend (experimental, requires `--features llvm`)
- Very small language subset: `print "..."`, `let x = input()`, arithmetic, conditionals, loops
- Direct object/ELF generation using the `object` crate, system `ld`/`objcopy`, or LLVM IR + gcc

Limitations & roadmap
- This is a tiny prototype for x86_64 Linux only.
- The final Wheel language you described (thousands of features, OS-level frameworks, installers, PATH integration, multi-arch, full toolchain) is a large project. This repo sets the architecture and an initial generator path that does not emit C

Quick start

Build:
```bash
cargo build --release
```

Build with LLVM backend (experimental):
```bash
cargo build --release --features llvm
```

Generate executable from the example (default backend):
```bash
./target/release/wheelc examples/hello.wheel -o hello --mode ge
./hello
```

Generate executable from the example (LLVM backend):
```bash
./target/release/wheelc examples/perguntas.wheel -o perguntas --mode ll
./perguntas
```

Generate flat binary:
```bash
./target/release/wheelc examples/hello.wheel -o hello.bin --mode gb
```

Install (copies the built `wheelc` to `/usr/local/bin`):
```bash
./scripts/install.sh
```

## LLVM Backend (Experimental)

The LLVM backend is now available as an experimental feature. It generates native code through LLVM IR, supporting:

**Supported Features:**
- Variable declarations: `let x = ...`
- String input: `let name = input()`
- Arithmetic operations: `+`, `-`, `*`, `/`, `%`
- String literals and print statements
- Conditionals: `if ... { ... }` and `if ... { ... } else { ... }`
- Loops: `while ... { ... }`
- C library integration: `printf`, `scanf`, `atoi` for type conversions

**Limitations:**
- Still uses `gcc` for final linking (not fully standalone LLVM)
- Unicode identifiers split during parsing (use ASCII names in source files)
- Strings internally represented as i64 pointers (type-unsafe, but functional)
- No function definitions yet (only built-in `input()`)
- No floating-point support (integers only)

**Known Issues & Workarounds:**
- Parser splits accented characters in identifiers: use ASCII-only names (e.g., `Opcao` instead of `Opçao`)
- Input results are pointer values cast to i64; arithmetic operations automatically convert via `atoi()`
- String comparisons convert both sides via `atoi()` before numeric comparison (avoids null pointer issues)

**Compilation & Testing:**
```bash
# Compile a Wheel program with LLVM backend
cargo run --release --features llvm -- examples/perguntas.wheel -o perguntas --mode ll

# Run interactively
./perguntas

# Or with piped input
printf "1\n5\n3\n" | ./perguntas

# Inspect generated LLVM IR
cat tmp.ll  # Contains the LLVM intermediate representation
```

**Internals:**
- LLVM IR generation: `src/llvm_backend.rs` (~510 lines)
- Global buffer for `scanf` input: 256-byte character array
- C function declarations: `printf`, `scanf`, `malloc`, `atoi`, `strcmp`
- Format strings: `"%ld\n"` for integers, `"%s\n"` for strings, `"%255s"` for input

Next steps
- ✓ Expand standard library (4 libraries implemented: SDL, Hardware I/O, Math, OS/Syscalls)
- [ ] Extend LLVM backend to support library function calls from Wheel
- [ ] Add f64 floating-point type support
- [ ] Fix input buffer handling (newline stripping)
- [ ] Expand parser, type system, and packages
- [ ] Add multi-architecture support and toolchain that can emit native ELF/PE/Mach-O directly
- [ ] Provide rich stdlib for OS development, GUI, game development, frameworks and automation

## Standard Libraries (NEW!)

Four comprehensive standard libraries have been implemented:

### 1. **SDL Library** (Graphics & Windowing)
- Window creation, rendering, event handling
- Ready for Wheel integration
- Feature flag: `sdl`
- See: `LIBRARIES.md`

### 2. **Hardware I/O Library** (Low-Level)
- CPU control (cli/sti), port I/O, memory access
- Register management (CR0, CR3, RFLAGS)
- Perfect for kernel development
- Feature flag: `hwio`
- See: `LIBRARIES.md`

### 3. **Math Library** (Advanced Arithmetic)
- Trigonometric: sin, cos, tan, asin, acos, atan, atan2
- Exponential: exp, log, log10
- Power & roots: pow, sqrt
- Rounding: ceil, floor, round
- Constants: PI, E, TAU
- Feature flag: `math`
- See: `LIBRARIES.md`

### 4. **OS/Syscalls Library** (System Integration)
- Process management: fork, wait, exit, getpid, kill
- File I/O: open, close, read, write
- IPC: pipes, dup, dup2
- Environment: getenv, getcwd, chdir
- Timers: sleep, usleep, time_now
- Feature flag: `os`
- See: `LIBRARIES.md`

**Building with Libraries:**
```bash
# All libraries
cargo build --release --features "llvm,sdl,hwio,math,os"

# Just core + hardware
cargo build --release --features "llvm,hwio"

# See LIBRARIES.md for full documentation
```

**Testing:**
```bash
# Compile integer arithmetic example (working!)
cargo run --release --features llvm -- examples/integer_test.wheel -o itest --mode ll
./itest

# Output: Correct arithmetic (15+25=40, etc.)
```

**Documentation:**
- `LIBRARIES.md` - Complete API reference for all 4 libraries
- `LIBS_ARCHITECTURE.md` - Detailed architecture and design
- `IMPLEMENTATION_STATUS.md` - Current status and known issues
# Wheel