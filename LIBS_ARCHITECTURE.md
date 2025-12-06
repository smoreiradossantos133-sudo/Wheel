# Wheel Standard Libraries Architecture Plan

This document outlines the design and architecture for four comprehensive standard libraries for the Wheel language, enabling graphics, hardware control, advanced computation, and OS integration.

## Overview

### Phase 1: Foundation (Q1)
- Library infrastructure: module system, C FFI bindings
- SDL library (graphics/windowing)
- Basic hardware I/O library

### Phase 2: Advanced (Q2)
- Advanced math library (floating-point, trigonometry, linear algebra)
- OS communication library (syscalls, signals, processes)

### Phase 3: Integration (Q3+)
- Complete kernel development toolkit
- Cross-platform support (Linux → Windows, macOS)

---

## 1. SDL Library (Graphics & Windowing)

**Purpose**: Enable graphics programming, game development, and GUI creation in Wheel

**Architecture**:
- **Language Binding**: Rust wrapper around SDL2-sys; exposed to Wheel via LLVM extern declarations
- **Implementation Path**: 
  1. Create `src/stdlib/sdl.rs` with SDL2 function declarations
  2. Generate LLVM IR stubs for Wheel that call Rust SDL wrappers
  3. Link compiled Wheel object files with SDL2 libraries
- **Key Components**:
  - Window creation: `create_window(width, height, title)`
  - Event handling: `poll_event()` → returns event type/code
  - Rendering: `draw_pixel(x, y, color)`, `draw_rect(x, y, w, h, color)`, `draw_line(x1, y1, x2, y2, color)`
  - Text rendering: `draw_text(x, y, text, size, color)`
  - Input: `get_key_state()` → keyboard state bitmask or poll result

**Initial Scope**:
```wheel
// Example Wheel code using SDL
let window = sdl_create_window(800, 600, "Game");
let running = 1;
while (running == 1) {
    let event = sdl_poll_event();
    if (event == QUIT_EVENT) {
        running = 0;
    }
    // Drawing code
}
sdl_destroy_window(window);
```

**Implementation Plan**:
1. Add `sdl2` crate to `Cargo.toml` as optional feature: `"sdl" = ["sdl2"]`
2. Create `src/stdlib/sdl.rs` with:
   - SDL context initialization/cleanup
   - Window/renderer management (thread-local or static)
   - Pixel buffer operations
   - Event queue wrapper
3. Extend LLVM backend to:
   - Declare SDL functions as extern C
   - Generate calls to C-compatible SDL wrapper functions
4. Create example: `examples/sdl_demo.wheel` (draw bouncing ball, handle window close)

**Dependencies**:
- `sdl2 = { version = "0.36", optional = true }`
- System SDL2 dev libraries: `libsdl2-dev`

**Limitations (v1)**:
- Single window only
- Basic 2D rendering only (no 3D/OpenGL)
- Synchronous event polling only

---

## 2. Hardware Communication Library (Low-Level I/O & Kernel Dev)

**Purpose**: Enable direct hardware access for kernel development, bare-metal programming, and low-level I/O control

**Architecture**:
- **Approach**: Unsafe Rust wrappers around x86_64 inline assembly
- **Implementation Path**:
  1. Create `src/stdlib/hw.rs` with hardware access functions
  2. Generate LLVM stubs that call Rust hardware wrappers via FFI
  3. Expose via optional feature `"hwio"`
- **Key Components**:

### CPU/x86_64 Operations
- `cpu_cli()` / `cpu_sti()` – disable/enable interrupts
- `cpu_hlt()` – halt CPU
- `cpu_nop()` – no-operation instruction
- `cpu_pause()` – pause (for spinloops)
- `cpu_get_cr0()` / `cpu_set_cr0(value)` – control register access
- `cpu_get_rflags()` / `cpu_set_rflags(value)` – flags register

### Memory Management
- `mem_read_byte(addr)` – read byte from address
- `mem_write_byte(addr, value)` – write byte to address
- `mem_read_word(addr)` – read 16-bit word
- `mem_write_word(addr, value)` – write 16-bit word
- `mem_read_dword(addr)` – read 32-bit dword
- `mem_write_dword(addr, value)` – write 32-bit dword
- `mem_read_qword(addr)` – read 64-bit qword
- `mem_write_qword(addr, value)` – write 64-bit qword

### I/O Port Operations
- `port_read_byte(port)` – read byte from I/O port
- `port_write_byte(port, value)` – write byte to I/O port
- `port_read_word(port)` – read 16-bit word from I/O port
- `port_write_word(port, value)` – write 16-bit word to I/O port

### GDT/IDT Management
- `gdt_load(addr)` – load GDT
- `idt_load(addr)` – load IDT
- `tss_set_stack(addr)` – set task stack pointer

### Paging
- `page_enable()` – enable paging
- `page_disable()` – disable paging
- `page_get_cr3()` / `page_set_cr3(cr3_value)` – page directory management

**Initial Scope**:
```wheel
// Example: Simple port read/write
let status = port_read_byte(0x3F8);  // Serial port status
if (status & 0x20) {
    port_write_byte(0x3F8, 65);  // Write 'A' to serial port
}

// Example: Disable interrupts during critical section
cpu_cli();
// ... critical code ...
cpu_sti();
```

**Implementation Plan**:
1. Add feature to `Cargo.toml`: `"hwio" = []`
2. Create `src/stdlib/hw.rs` with inline assembly wrappers:
   ```rust
   pub unsafe fn cpu_cli() {
       asm!("cli", options(noreturn));
   }
   pub unsafe fn port_read_byte(port: u16) -> u8 {
       let value: u8;
       asm!("in al, dx", in("dx") port, out("al") value);
       value
   }
   ```
3. Extend LLVM backend with `hwio` feature support:
   - Declare functions as `unsafe` externals
   - Generate calls with inline asm where needed
4. Create example: `examples/kernel_simple.wheel` (serial port echo)

**Dependencies**:
- Inline assembly (no external crates needed)
- x86_64 architecture (initially)
- Kernel-level privileges (for actual hardware access)

**Limitations (v1)**:
- x86_64 Linux only
- Requires elevated privileges for real I/O
- No interrupt handler registration yet
- Single-threaded only

---

## 3. Advanced Math Library (Floating-Point & Transcendental Functions)

**Purpose**: Enable scientific computing, graphics transformations, and advanced calculations beyond integer arithmetic

**Architecture**:
- **Implementation**: Rust wrapper around libm (math.h) functions
- **Approach**: Extend LLVM backend to support f64 type, declare C math library functions
- **Key Components**:

### Floating-Point Basics
- `fsin(x)`, `fcos(x)`, `ftan(x)` – trigonometric functions
- `fasin(x)`, `facos(x)`, `fatan(x)` – inverse trigonometric
- `fatan2(y, x)` – two-argument arctangent
- `fsqrt(x)` – square root
- `fpow(x, y)` – power function
- `fexp(x)`, `flog(x)`, `flog10(x)` – exponential/logarithmic
- `fceil(x)`, `ffloor(x)`, `fround(x)` – rounding functions
- `fabs(x)` – absolute value
- `fmod(x, y)` – floating-point modulo
- `fmin(x, y)`, `fmax(x, y)` – minimum/maximum

### Linear Algebra (Matrix/Vector Operations)
- `matrix_create(rows, cols)` → handle
- `matrix_set(handle, row, col, value)`
- `matrix_get(handle, row, col)` → value
- `matrix_multiply(a_handle, b_handle)` → result_handle
- `matrix_transpose(handle)` → result_handle
- `matrix_determinant(handle)` → value
- `matrix_inverse(handle)` → result_handle
- `vector_dot(v1, v2)` → scalar
- `vector_cross(v1, v2)` → vector_handle

### Constants
- `PI`, `E`, `TAU` – mathematical constants

**Initial Scope**:
```wheel
// Example: Graphics transformation
let angle = 45;
let radians = (angle * PI) / 180;
let rotated_x = (x * fcos(radians)) - (y * fsin(radians));
let rotated_y = (x * fsin(radians)) + (y * fcos(radians));

// Example: Physics calculation
let velocity = 9.8;  // m/s^2
let time = 2.0;
let distance = 0.5 * velocity * (time * time);
```

**Implementation Plan**:
1. Extend `src/llvm_backend.rs` to:
   - Support `f64` type alongside `i64`
   - Declare `libm` functions: `sin`, `cos`, `tan`, `sqrt`, `pow`, `exp`, `log`, etc.
   - Generate LLVM floating-point operations
2. Create `src/stdlib/math.rs` (enhancement to existing math.rs):
   - Matrix/vector operations (may use external `nalgebra` crate or implement from scratch)
   - Allocators for matrix handles
3. Create example: `examples/math_graphics.wheel` (simple 3D rotation)

**Dependencies**:
- libm (C standard math library – always available on Linux)
- Optional: `nalgebra = "0.33"` for advanced linear algebra

**Limitations (v1)**:
- No complex numbers
- No symbolic math
- Matrix operations require explicit allocation/deallocation
- Single precision or double precision only

---

## 4. OS/Software Communication Library (Syscalls & Processes)

**Purpose**: Enable system-level programming, process management, IPC, and OS interaction

**Architecture**:
- **Approach**: Direct syscall numbers + inline asm (Linux x86_64); abstraction layer for portability
- **Implementation Path**:
  1. Create `src/stdlib/os.rs` with syscall wrappers
  2. Generate LLVM stubs that call Rust syscall wrappers
  3. Handle Linux syscall convention (rax = syscall number; rdi, rsi, rdx, r10, r8, r9 = args; rax = return)
- **Key Components**:

### Process Management
- `exec(path, args)` – execute program
- `fork()` – duplicate process
- `wait(pid)` – wait for child process
- `exit(code)` – exit process with code
- `getpid()` – get current process ID
- `kill(pid, signal)` – send signal to process

### File I/O (extend existing)
- `open(path, flags)` → fd
- `close(fd)`
- `read(fd, buffer, size)` → bytes_read
- `write(fd, buffer, size)` → bytes_written
- `seek(fd, offset, whence)` → new_position
- `stat(path)` → size, mode, mtime

### IPC & Signals
- `signal(signum, handler)` – register signal handler
- `sigaction(signum, action)` – advanced signal setup
- `pipe(read_fd, write_fd)` – create pipe
- `socket(domain, type, protocol)` → sock_fd
- `bind(sock, address, port)`
- `listen(sock, backlog)`
- `accept(sock)` → client_fd
- `connect(sock, address, port)`
- `send(sock, data)` → bytes_sent
- `recv(sock, buffer)` → bytes_received

### Environment & Shell
- `getenv(name)` → value (string pointer)
- `setenv(name, value)`
- `getcwd()` → path (string pointer)
- `chdir(path)`
- `system(command)` → exit_code
- `popen(command)` → pipe_fd (for command output)

### Timers & Scheduling
- `sleep(seconds)`
- `usleep(microseconds)`
- `clock_gettime()` → nanoseconds since epoch
- `setitimer(seconds, callback)` – set recurring timer

**Initial Scope**:
```wheel
// Example: Child process with signal handling
let pid = fork();
if (pid == 0) {
    // Child process
    print "Child started\n";
} else {
    // Parent process
    print "Parent waiting\n";
    wait(pid);
    print "Child finished\n";
}

// Example: Simple TCP server
let sock = socket(2, 1, 0);  // AF_INET, SOCK_STREAM, IPPROTO_TCP
bind(sock, 127.0.0.1, 8080);
listen(sock, 5);
let client = accept(sock);
// ... read/write client ...
```

**Implementation Plan**:
1. Extend/refactor `src/stdlib/os.rs` (currently stubbed in Cargo.toml):
   - Syscall dispatch using inline asm for x86_64 Linux
   - Wrapper functions for each syscall
   - Error handling (check for negative return values)
2. Create syscall number constants (e.g., `SYS_OPEN = 2`, `SYS_READ = 0`, etc.)
3. Extend LLVM backend to:
   - Declare OS functions as extern C
   - Generate calls to Rust wrappers
4. Create example: `examples/os_server.wheel` (simple TCP echo server)

**Dependencies**:
- libc = "0.2" (for syscall numbers and type definitions)
- Optional: `async-std` or `tokio` for async I/O (future)

**Limitations (v1)**:
- Linux x86_64 only (syscall numbers differ on other architectures/OSes)
- No async I/O (synchronous syscalls only)
- No thread creation yet
- Error handling: return negative values or special codes

---

## Integration Strategy

### Cargo Features
```toml
[features]
default = []
llvm = ["inkwell/llvm16-0-no-llvm-linking"]
sdl = ["sdl2"]
hwio = []
math = ["libm"]  # optional, libm usually linked by default
os = []

[dependencies]
sdl2 = { version = "0.36", optional = true }
libc = "0.2"
```

### LLVM Backend Extensions
1. Add feature-gated function declarations in `src/llvm_backend.rs`:
   ```rust
   #[cfg(feature = "sdl")]
   fn declare_sdl_functions(...) { ... }
   
   #[cfg(feature = "hwio")]
   fn declare_hw_functions(...) { ... }
   ```

2. Extend `gen_expr` to recognize library calls:
   - `Expr::Call { name: "sdl_create_window", args: [...] }` → SDL2 extern call
   - `Expr::Call { name: "port_read_byte", args: [...] }` → hardware I/O call
   - `Expr::Call { name: "fsin", args: [...] }` → math library call
   - `Expr::Call { name: "fork", args: [...] }` → OS syscall wrapper

### Compilation Flow
```
Wheel source → Parser → AST → LLVM IR Generator
                                   ↓ (with features enabled)
                           Declares SDL2/HW/Math/OS functions
                                   ↓
                           LLVM IR → Object file (.o)
                                   ↓
                           gcc/clang linker
                                   ↓
                           Executable (linked with SDL2/libc/etc)
```

### Library Initialization & Cleanup
- **SDL**: Auto-init on first window creation, cleanup on exit
- **Hardware**: No init needed (unsafe operations); must be called from privileged context
- **Math**: No init; uses standard libm
- **OS**: No init; direct syscalls; signal handlers optional

---

## Development Timeline

### Phase 1 (Sprint 1-2)
1. **Weeks 1-2**: SDL library foundation
   - Add SDL2 bindings, basic window/rendering functions
   - Example: `sdl_demo.wheel` (draw colored squares)
   - Milestone: Graphics window opens and displays content

### Phase 2 (Sprint 3-4)
2. **Weeks 3-4**: Hardware I/O library
   - Port read/write operations, CPU control
   - Example: `hw_serial_echo.wheel` (serial port communication)
   - Milestone: Low-level I/O functions callable from Wheel

### Phase 3 (Sprint 5-6)
3. **Weeks 5-6**: Advanced math library
   - Floating-point type support in LLVM backend (major refactor)
   - Trigonometric/exponential functions
   - Example: `math_graphics.wheel` (rotating 3D wireframe)
   - Milestone: f64 type works; math functions callable

### Phase 4 (Sprint 7-8)
4. **Weeks 7-8**: OS communication library
   - Syscall wrappers, process/file operations
   - Example: `os_server.wheel` (HTTP server or file processor)
   - Milestone: Fork, pipes, sockets work; data exchanged with OS

---

## Testing Strategy

### Unit Tests
- Each library has test module in `src/stdlib/lib_name.rs`
- Example functions tested in isolation with known inputs/outputs

### Integration Tests
- `tests/integration_test.rs` extended with library tests
- Test each library against multiple Wheel examples

### Example Programs
- **SDL**: `examples/sdl_draw.wheel`, `examples/sdl_game.wheel`
- **Hardware**: `examples/hw_ports.wheel`, `examples/hw_cpu.wheel`
- **Math**: `examples/math_mandelbrot.wheel`, `examples/math_physics.wheel`
- **OS**: `examples/os_fork.wheel`, `examples/os_server.wheel`

---

## Future Enhancements (Phase 2+)

1. **Cross-Platform Support**:
   - Abstract syscalls behind platform layer
   - Support Windows (native API) and macOS (different syscall numbers)

2. **Async I/O**:
   - Event loop integration for SDL and network I/O
   - Coroutine/green thread support

3. **Type Safety**:
   - Struct types for SDL resources, file handles, sockets
   - Move semantics for automatic cleanup

4. **Advanced Graphics**:
   - OpenGL/Vulkan integration
   - 3D transformations and rendering

5. **Real-Time Computing**:
   - RTIC integration for deterministic scheduling
   - RT-level timing guarantees

6. **Kernel Development Kit**:
   - Pre-built bootloader + Wheel kernel template
   - Bare-metal linking and memory layout tools

---

## Success Criteria

By end of Phase 1:
- [ ] SDL library implemented; graphics window opens from Wheel code
- [ ] Hardware library accessible; port I/O functions callable
- [ ] Math library foundation; floating-point type supported
- [ ] OS library; fork/wait operations functional
- [ ] All 4 examples compile and run successfully
- [ ] README documentation updated for each library
- [ ] Features testable via `cargo build --features "sdl,hwio,math,os"`

