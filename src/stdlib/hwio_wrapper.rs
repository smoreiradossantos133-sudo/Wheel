//! Hardware I/O and CPU control for Wheel
//! Provides low-level access for kernel development and hardware interaction

#[cfg(feature = "hwio")]
pub mod hwio {
    use core::arch::asm;

    /// Disable interrupts
    pub fn cpu_cli() -> i64 {
        unsafe { asm!("cli"); }
        1
    }

    /// Enable interrupts
    pub fn cpu_sti() -> i64 {
        unsafe { asm!("sti"); }
        1
    }

    /// Halt CPU
    pub fn cpu_hlt() -> i64 {
        unsafe { asm!("hlt"); }
        0 // Unreachable, but Rust type system requires return
    }

    /// No-operation instruction
    pub fn cpu_nop() -> i64 {
        unsafe { asm!("nop"); }
        1
    }

    /// Pause for spin-loops
    pub fn cpu_pause() -> i64 {
        unsafe { asm!("pause"); }
        1
    }

    /// Read byte from I/O port
    pub fn port_read_byte(port: u16) -> i64 {
        let value: u8;
        unsafe {
            asm!(
                "in al, dx",
                in("dx") port,
                out("al") value,
                options(nostack, nomem)
            );
        }
        value as i64
    }

    /// Write byte to I/O port
    pub fn port_write_byte(port: u16, value: u8) -> i64 {
        unsafe {
            asm!(
                "out dx, al",
                in("dx") port,
                in("al") value,
                options(nostack, nomem)
            );
        }
        1
    }

    /// Read word (16-bit) from I/O port
    pub fn port_read_word(port: u16) -> i64 {
        let value: u16;
        unsafe {
            asm!(
                "in ax, dx",
                in("dx") port,
                out("ax") value,
                options(nostack, nomem)
            );
        }
        value as i64
    }

    /// Write word (16-bit) to I/O port
    pub fn port_write_word(port: u16, value: u16) -> i64 {
        unsafe {
            asm!(
                "out dx, ax",
                in("dx") port,
                in("ax") value,
                options(nostack, nomem)
            );
        }
        1
    }

    /// Read from memory address (byte)
    pub fn mem_read_byte(addr: i64) -> i64 {
        let ptr = addr as *const u8;
        unsafe { *ptr as i64 }
    }

    /// Write to memory address (byte)
    pub fn mem_write_byte(addr: i64, value: u8) -> i64 {
        let ptr = addr as *mut u8;
        unsafe { *ptr = value; }
        1
    }

    /// Read from memory address (32-bit)
    pub fn mem_read_dword(addr: i64) -> i64 {
        let ptr = addr as *const u32;
        unsafe { *ptr as i64 }
    }

    /// Write to memory address (32-bit)
    pub fn mem_write_dword(addr: i64, value: u32) -> i64 {
        let ptr = addr as *mut u32;
        unsafe { *ptr = value; }
        1
    }

    /// Read from memory address (64-bit)
    pub fn mem_read_qword(addr: i64) -> i64 {
        let ptr = addr as *const i64;
        unsafe { *ptr }
    }

    /// Write to memory address (64-bit)
    pub fn mem_write_qword(addr: i64, value: i64) -> i64 {
        let ptr = addr as *mut i64;
        unsafe { *ptr = value; }
        1
    }

    /// Get CR0 register
    pub fn cpu_get_cr0() -> i64 {
        let value: u64;
        unsafe {
            asm!("mov {}, cr0", out(reg) value);
        }
        value as i64
    }

    /// Set CR0 register
    pub fn cpu_set_cr0(value: u64) -> i64 {
        unsafe {
            asm!("mov cr0, {}", in(reg) value);
        }
        1
    }

    /// Get CR3 register (page directory base)
    pub fn cpu_get_cr3() -> i64 {
        let value: u64;
        unsafe {
            asm!("mov {}, cr3", out(reg) value);
        }
        value as i64
    }

    /// Set CR3 register (page directory base)
    pub fn cpu_set_cr3(value: u64) -> i64 {
        unsafe {
            asm!("mov cr3, {}", in(reg) value);
        }
        1
    }

    /// Get RFLAGS register
    pub fn cpu_get_rflags() -> i64 {
        let value: u64;
        unsafe {
            asm!("pushfq; pop {}", out(reg) value);
        }
        value as i64
    }

    /// Set RFLAGS register
    pub fn cpu_set_rflags(value: u64) -> i64 {
        unsafe {
            asm!("push {}; popfq", in(reg) value);
        }
        1
    }
}

#[cfg(not(feature = "hwio"))]
pub mod hwio {
    pub fn cpu_cli() -> i64 { -1 }
    pub fn cpu_sti() -> i64 { -1 }
    pub fn cpu_hlt() -> i64 { -1 }
    pub fn cpu_nop() -> i64 { -1 }
    pub fn cpu_pause() -> i64 { -1 }
    pub fn port_read_byte(_p: u16) -> i64 { -1 }
    pub fn port_write_byte(_p: u16, _v: u8) -> i64 { -1 }
    pub fn port_read_word(_p: u16) -> i64 { -1 }
    pub fn port_write_word(_p: u16, _v: u16) -> i64 { -1 }
    pub fn mem_read_byte(_a: i64) -> i64 { -1 }
    pub fn mem_write_byte(_a: i64, _v: u8) -> i64 { -1 }
    pub fn mem_read_dword(_a: i64) -> i64 { -1 }
    pub fn mem_write_dword(_a: i64, _v: u32) -> i64 { -1 }
    pub fn mem_read_qword(_a: i64) -> i64 { -1 }
    pub fn mem_write_qword(_a: i64, _v: i64) -> i64 { -1 }
    pub fn cpu_get_cr0() -> i64 { -1 }
    pub fn cpu_set_cr0(_v: u64) -> i64 { -1 }
    pub fn cpu_get_cr3() -> i64 { -1 }
    pub fn cpu_set_cr3(_v: u64) -> i64 { -1 }
    pub fn cpu_get_rflags() -> i64 { -1 }
    pub fn cpu_set_rflags(_v: u64) -> i64 { -1 }
}
