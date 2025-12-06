# Wheel Standard Libraries

This document describes the four new standard libraries available in Wheel, along with practical examples.

## Quick Start

### Building with Libraries

```bash
# Build with all features
cargo build --release --features "llvm,sdl,hwio,math,os"

# Build with specific features
cargo build --release --features "llvm,math,os"
```

### Compiling Wheel Programs

```bash
# Compile a Wheel program
cargo run --release --features llvm -- examples/program.wheel -o program --mode ll

# Run the compiled program
./program
```

---

## 1. SDL Library (Graphics)

**Status**: Implemented with bindings ready  
**Features**: Window creation, pixel drawing, rendering, event handling

### Wheel API (Coming Soon)

Once full FFI support is integrated into the LLVM backend:

```wheel
// Create a window
let window = sdl_init();
let result = sdl_create_window(800, 600, "My Window");

// Draw pixels
sdl_draw_pixel(100, 100, 255, 0, 0);    // Red pixel
sdl_draw_rect(50, 50, 200, 200, 0, 255, 0);  // Green rectangle

// Clear and present
sdl_clear(0, 0, 255);  // Blue background
sdl_present();

// Cleanup
sdl_destroy_window();
sdl_quit();
```

### Example Program

See: `examples/sdl_demo.wheel` (placeholder - full support coming)

### Under the Hood

- **Rust Module**: `src/stdlib/sdl_wrapper.rs`
- **Dependency**: `sdl2 = "0.36"`
- **Integration**: Cargo feature `sdl` pulls in SDL2 bindings

**Limitations (v1)**:
- Single window only
- 2D rendering only
- Synchronous event polling

---

## 2. Hardware I/O Library (Low-Level)

**Status**: Implemented with inline assembly  
**Features**: CPU control, port I/O, memory access, register management

### Wheel API (For Kernel Development)

Once integrated into LLVM backend:

```wheel
// CPU Operations
cpu_cli();   // Disable interrupts
cpu_sti();   // Enable interrupts
cpu_hlt();   // Halt CPU
cpu_nop();   // No-op instruction

// I/O Port Operations
let status = port_read_byte(0x3F8);     // Read from serial port
port_write_byte(0x3F8, 65);              // Write to serial port
let word = port_read_word(0x1F0);        // Read word from IDE controller

// Memory Operations
let value = mem_read_byte(0x400000);     // Read byte from memory
mem_write_byte(0x400000, 42);            // Write byte to memory
let qword = mem_read_qword(0x500000);    // Read 64-bit value

// Control Registers
let cr0 = cpu_get_cr0();                 // Get CR0 (control register)
cpu_set_cr0(cr0 | 0x1);                  // Enable paging
let cr3 = cpu_get_cr3();                 // Get page directory base
cpu_set_cr3(new_page_dir);               // Switch page tables
```

### Example Program

See: `examples/hw_demo.wheel`

### Under the Hood

- **Rust Module**: `src/stdlib/hwio_wrapper.rs`
- **Implementation**: x86_64 inline assembly (no external dependencies)
- **Cargo Feature**: `hwio`
- **Privilege Required**: Kernel mode or elevated privileges for actual I/O

**Limitations (v1)**:
- x86_64 Linux only
- Requires elevated privileges for real hardware access
- No interrupt handler registration yet

---

## 3. Math Library (Advanced Arithmetic)

**Status**: Rust wrapper ready; LLVM backend f64 support pending  
**Features**: Trigonometry, exponential, logarithmic, linear algebra

### Wheel API (Coming Soon with f64 support)

```wheel
// Trigonometric functions (radians)
let sine = math_sin(3.14159 / 2.0);      // sin(π/2) = 1.0
let cosine = math_cos(0.0);              // cos(0) = 1.0
let tangent = math_tan(0.785398);        // tan(π/4) ≈ 1.0

// Inverse trigonometric
let angle = math_asin(0.5);              // asin(0.5) = π/6
let angle2 = math_atan2(1.0, 1.0);       // atan2(1, 1) = π/4

// Exponential and logarithmic
let ex = math_exp(1.0);                  // e ≈ 2.71828
let lnx = math_log(2.71828);             // ln(e) = 1.0
let log10x = math_log10(100.0);          // log₁₀(100) = 2.0

// Power and roots
let squared = math_pow(5.0, 2.0);        // 5² = 25
let root = math_sqrt(16.0);              // √16 = 4

// Rounding functions
let rounded = math_round(3.7);           // 4
let floored = math_floor(3.7);           // 3
let ceiled = math_ceil(3.2);             // 4

// Utility functions
let absolute = math_abs(-42.5);          // 42.5
let minimum = math_min(5.5, 3.2);        // 3.2
let maximum = math_max(5.5, 3.2);        // 5.5

// Constants
let pi = PI;                             // 3.14159...
let e = E;                               // 2.71828...
let tau = TAU;                           // 6.28318...
```

### Example Program

See: `examples/math_calc.wheel`

### Under the Hood

- **Rust Module**: `src/stdlib/math_wrapper.rs`
- **Functions**: Wrappers around Rust's f64 math library
- **Cargo Feature**: `math`
- **Backend Change**: Requires LLVM f64 type support in `src/llvm_backend.rs`

**Limitations (v1)**:
- No matrix operations yet
- Requires LLVM backend to support f64 type
- No complex number support

---

## 4. OS/Software Communication Library

**Status**: Syscall wrappers implemented  
**Features**: Process management, file I/O, IPC, signals, timers

### Wheel API

Some functions already work:

```wheel
// Process Management
let my_pid = getpid();                   // Get current process ID (WORKS)
print(my_pid);

// Coming Soon:
// let child_pid = fork();               // Create child process
// if (child_pid == 0) {
//     // Child process code
// } else {
//     wait(child_pid);                  // Wait for child
// }
// exit(0);                              // Exit process with code

// kill(pid, SIGTERM);                   // Send signal to process

// File I/O (partial support via stdlib improvements)
// let fd = open("/tmp/file.txt", 0);    // Open file (O_RDONLY = 0)
// let bytes = read(fd, buf, 256);       // Read from file
// write(fd, buf, 256);                  // Write to file
// close(fd);                            // Close file

// Pipes and IPC
// pipe(read_fd, write_fd);              // Create pipe
// dup2(fd1, fd2);                       // Duplicate file descriptor

// Environment
// let path = getenv("PATH");            // Get environment variable
// setenv("VAR", "value");               // Set environment variable
// let cwd = getcwd(buffer, 256);        // Get current directory
// chdir("/tmp");                        // Change directory

// System Operations
// system("ls -la");                     // Execute shell command
// sleep(5);                             // Sleep 5 seconds
// usleep(500000);                       // Sleep 500ms
// let timestamp = time_now();           // Get seconds since epoch
```

### Example Program

See: `examples/os_demo.wheel`

### Under the Hood

- **Rust Module**: `src/stdlib/os_wrapper.rs`
- **Implementation**: Direct syscall wrappers for Linux x86_64
- **Dependency**: `libc = "0.2"`
- **Cargo Feature**: `os`

**Limitations (v1)**:
- Linux x86_64 only (syscall numbers differ on other architectures)
- No async I/O support
- No thread creation yet
- Error handling: negative return values or specific codes

---

## Working Example: Full Calculator

This example uses core Wheel features that already work:

```bash
cargo run --release --features llvm -- examples/full_calculator.wheel -o calc --mode ll
echo "1" | echo "5" | echo "3" | ./calc
```

**Features Demonstrated**:
- Menu loops
- Integer arithmetic (+, -, *, /, %)
- String input/output
- Conditional branching
- Variable assignment

**Output**:
```
╔════════════════════════════════════╗
║   Calculadora Wheel v1.0           ║
╚════════════════════════════════════╝

Escolha uma operacao:
1: Adicao
2: Subtracao
...
```

---

## Integration Roadmap

### Phase 1 (Current)
- ✓ SDL bindings created
- ✓ Hardware I/O inline assembly ready
- ✓ Math functions wrapped
- ✓ OS syscalls available
- [ ] Full LLVM backend integration
- [ ] Library function calls in Wheel AST

### Phase 2 (Next)
- [ ] Extend LLVM backend to support library calls
- [ ] Add f64 type support to backend
- [ ] Create SDL window rendering example
- [ ] Demonstrate hardware I/O (kernel context)

### Phase 3 (Future)
- [ ] Cross-platform support (Windows, macOS)
- [ ] Async I/O integration
- [ ] Type-safe resource management
- [ ] Complete kernel development toolkit

---

## Testing Libraries

Each library is tested independently:

```bash
# Test with SDL enabled
cargo test --features sdl

# Test with hardware features
cargo test --features hwio

# Test math functions
cargo test --features math

# Test all
cargo test --all-features
```

---

## Debugging

### Check Generated LLVM IR

After compilation, inspect the LLVM intermediate representation:

```bash
cat tmp.ll | head -100  # View first 100 lines
```

### Trace Function Calls

Enable debug output in Rust code:

```rust
eprintln!("Calling sdl_create_window");
```

---

## Next Steps

1. **For Graphics**: Extend LLVM backend to declare SDL2 functions
2. **For Hardware**: Create bare-metal kernel example
3. **For Math**: Add f64 type support to LLVM backend
4. **For OS**: Implement fork/wait/pipe operations

See `LIBS_ARCHITECTURE.md` for detailed technical design.
