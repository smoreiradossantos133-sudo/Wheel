# Wheel Libraries - Quick Reference Guide

## Building

### With All Features
```bash
cargo build --release --features "llvm,sdl,hwio,math,os"
```

### With Specific Features
```bash
# LLVM backend only (core)
cargo build --release --features llvm

# LLVM + Hardware I/O (for kernel dev)
cargo build --release --features "llvm,hwio"

# LLVM + Math (for scientific computing)
cargo build --release --features "llvm,math"

# LLVM + OS syscalls (for system programming)
cargo build --release --features "llvm,os"

# LLVM + SDL (for graphics)
cargo build --release --features "llvm,sdl"
```

---

## Quick Examples

### 1. Integer Arithmetic (Working Now!)
```bash
cargo run --release --features llvm -- examples/integer_test.wheel -o itest --mode ll
./itest
```

**Output:**
```
=== Integer Literal Test ===
15 + 25 = 40
25 - 15 = 10
15 * 25 = 375
25 / 15 = 1
25 % 15 = 10
=== All tests completed ===
```

### 2. Simple Addition
```bash
cargo run --release --features llvm -- examples/simple_math.wheel -o smath --mode ll
printf "10\n20\n" | ./smath
```

**Output:**
```
Enter first number: 
Enter second number: 
Sum: 40
```

### 3. Menu Calculator
```bash
cargo run --release --features llvm -- examples/full_calculator.wheel -o calc --mode ll
# Run interactively: ./calc
```

---

## Library APIs

### SDL Library
```rust
// In Rust (already implemented)
// Wheel support coming soon after LLVM backend extension
use crate::stdlib::sdl_wrapper::sdl;

sdl::init();
sdl::create_window(800, 600, "Title");
sdl::draw_pixel(x, y, r, g, b);
sdl::draw_rect(x, y, w, h, r, g, b);
sdl::clear(r, g, b);
sdl::present();
sdl::destroy_window();
sdl::quit();
```

### Hardware I/O Library
```rust
// In Rust (already implemented)
// Wheel support coming soon
use crate::stdlib::hwio_wrapper::hwio;

hwio::cpu_cli();    // Disable interrupts
hwio::cpu_sti();    // Enable interrupts
let status = hwio::port_read_byte(0x3F8);
hwio::port_write_byte(0x3F8, 65);
let value = hwio::mem_read_byte(0x400000);
hwio::mem_write_byte(0x400000, 42);
```

### Math Library
```rust
// In Rust (already implemented)
// Wheel support coming soon (requires f64 LLVM backend support)
use crate::stdlib::math_wrapper::math;

let sine = math::math_sin(0.0);
let root = math::math_sqrt(16.0);
let power = math::math_pow(2.0, 8.0);
let pi = math::PI;
let e = math::E;
```

### OS Syscalls Library
```rust
// In Rust (already implemented)
// Partial Wheel support coming soon
use crate::stdlib::os_wrapper::os;

let pid = os::getpid();
let child_pid = os::fork();
os::wait(child_pid);
os::sleep(5);
let timestamp = os::time_now();
```

---

## File Organization

```
/workspaces/Wheel/
├── src/
│   ├── stdlib/
│   │   ├── mod.rs              # Library exports
│   │   ├── sdl_wrapper.rs      # SDL2 graphics library (90 lines)
│   │   ├── hwio_wrapper.rs     # Hardware I/O library (150 lines)
│   │   ├── math_wrapper.rs     # Math functions (120 lines)
│   │   └── os_wrapper.rs       # OS syscalls (200 lines)
│   ├── llvm_backend.rs         # LLVM IR generation
│   ├── main.rs                 # CLI entry point
│   └── ... other files
├── examples/
│   ├── integer_test.wheel      # ✓ Working - arithmetic test
│   ├── simple_math.wheel       # Basic addition
│   ├── full_calculator.wheel   # Menu-driven calculator
│   ├── hw_demo.wheel           # Hardware I/O example (placeholder)
│   ├── os_demo.wheel           # OS operations example
│   ├── math_calc.wheel         # Math functions example (placeholder)
│   └── ... other examples
├── Cargo.toml                  # Build config with features
├── README.md                   # Main documentation
├── LIBRARIES.md                # Library API reference
├── LIBS_ARCHITECTURE.md        # Design and planning
├── IMPLEMENTATION_STATUS.md    # Current status report
└── QUICKSTART.md               # This file!
```

---

## Known Limitations (v1)

1. **Input Buffer**: `scanf("%255s")` includes newline → `atoi()` adds 10
   - Workaround: Use integer literals instead of `input()`

2. **Escape Sequences**: `\n` prints literally, not as newline
   - Workaround: Use inline newlines in print statements

3. **Library Calls from Wheel**: SDL, hardware, math, OS libs not yet callable
   - **Status**: Rust implementations complete; LLVM backend extension needed
   - **Timeline**: Next development phase

4. **Modulo**: Some edge cases return incorrect values
   - Status: Under investigation

5. **Unicode**: Parser splits accented characters in identifiers
   - Workaround: Use ASCII-only names

---

## For Developers

### Build Everything
```bash
cargo build --release --all-features
```

### Run Tests
```bash
cargo test --all-features
```

### Check for Compilation Errors
```bash
cargo check --all-features
```

### Generate Documentation
```bash
cargo doc --all-features --open
```

### View Generated LLVM IR
```bash
cat tmp.ll | less
```

### Compile and Debug
```bash
# Compile to IR, object, then executable
cargo run --release --features llvm -- examples/integer_test.wheel -o itest --mode ll

# Inspect object file
objdump -d itest | head -100

# Run with strace to see syscalls
strace ./itest
```

---

## Next Steps

### Immediate (Phase 2)
- [ ] Fix input buffer newline handling
- [ ] Extend LLVM backend to support library calls
- [ ] Add f64 floating-point type
- [ ] Create working SDL demo
- [ ] Create hardware I/O kernel example
- [ ] Create math operations example

### Future (Phase 3)
- [ ] Cross-platform support (Windows, macOS)
- [ ] Async I/O and coroutines
- [ ] Type-safe resource management
- [ ] Complete kernel development toolkit
- [ ] Package manager and standard library

---

## Getting Help

1. **API Reference**: See `LIBRARIES.md`
2. **Architecture**: See `LIBS_ARCHITECTURE.md`
3. **Status**: See `IMPLEMENTATION_STATUS.md`
4. **Examples**: Browse `examples/` directory

---

## Summary

✓ **Done**: 4 complete standard libraries implemented in Rust  
✓ **Done**: Cargo integration with optional features  
✓ **Done**: Example programs (integer arithmetic working!)  
✓ **Done**: Comprehensive documentation  

⏳ **Next**: LLVM backend integration for library function calls  
⏳ **Next**: f64 floating-point type support  
⏳ **Next**: Input buffer fix  

**Status**: Ready for gradual integration into Wheel language!
