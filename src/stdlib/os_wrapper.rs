//! OS/Software Communication library for Wheel
//! Provides syscall wrappers, process management, file I/O, and IPC

#[cfg(target_os = "linux")]
pub mod os {
    use libc::{syscall, c_int, pid_t};

    // Syscall numbers for x86_64 Linux
    const SYS_READ: i64 = 0;
    const SYS_WRITE: i64 = 1;
    const SYS_OPEN: i64 = 2;
    const SYS_CLOSE: i64 = 3;
    const SYS_STAT: i64 = 4;
    const SYS_FORK: i64 = 57;
    const SYS_EXEC: i64 = 59;
    const SYS_EXIT: i64 = 60;
    const SYS_WAIT4: i64 = 114;
    const SYS_PIPE: i64 = 22;
    const SYS_GETPID: i64 = 39;
    const SYS_KILL: i64 = 62;
    const SYS_DUP: i64 = 32;
    const SYS_DUP2: i64 = 33;
    const SYS_FCNTL: i64 = 72;

    /// Fork current process
    pub fn fork() -> i64 {
        unsafe { syscall(SYS_FORK) as i64 }
    }

    /// Exit process with code
    pub fn exit(code: i32) -> i64 {
        unsafe { syscall(SYS_EXIT, code as i64); }
        -1 // Unreachable
    }

    /// Wait for child process
    pub fn wait(pid: i64) -> i64 {
        unsafe { syscall(SYS_WAIT4, pid, std::ptr::null_mut::<i32>(), 0, std::ptr::null_mut()) as i64 }
    }

    /// Get current process ID
    pub fn getpid() -> i64 {
        unsafe { syscall(SYS_GETPID) as i64 }
    }

    /// Send signal to process
    pub fn kill(pid: i64, sig: i32) -> i64 {
        unsafe { syscall(SYS_KILL, pid, sig as i64) as i64 }
    }

    /// Open file
    pub fn open(path_ptr: i64, flags: i32) -> i64 {
        let path = unsafe {
            let ptr = path_ptr as *const u8;
            std::ffi::CStr::from_ptr(ptr as *const i8)
                .to_string_lossy()
                .into_owned()
        };
        let c_path = std::ffi::CString::new(path).unwrap();
        unsafe { syscall(SYS_OPEN, c_path.as_ptr(), flags as i64, 0o644 as i64) as i64 }
    }

    /// Close file descriptor
    pub fn close(fd: i32) -> i64 {
        unsafe { syscall(SYS_CLOSE, fd as i64) as i64 }
    }

    /// Read from file descriptor
    pub fn read(fd: i32, buf_ptr: i64, count: usize) -> i64 {
        let buf = unsafe { std::slice::from_raw_parts_mut(buf_ptr as *mut u8, count) };
        unsafe { syscall(SYS_READ, fd as i64, buf.as_mut_ptr(), count as i64) as i64 }
    }

    /// Write to file descriptor
    pub fn write(fd: i32, buf_ptr: i64, count: usize) -> i64 {
        let buf = unsafe { std::slice::from_raw_parts(buf_ptr as *const u8, count) };
        unsafe { syscall(SYS_WRITE, fd as i64, buf.as_ptr(), count as i64) as i64 }
    }

    /// Create pipe
    pub fn pipe(read_fd: i64, write_fd: i64) -> i64 {
        let fds = unsafe { std::slice::from_raw_parts_mut(read_fd as *mut i32, 2) };
        unsafe { syscall(SYS_PIPE, fds.as_mut_ptr()) as i64 }
    }

    /// Duplicate file descriptor
    pub fn dup(oldfd: i32) -> i64 {
        unsafe { syscall(SYS_DUP, oldfd as i64) as i64 }
    }

    /// Duplicate file descriptor to specific number
    pub fn dup2(oldfd: i32, newfd: i32) -> i64 {
        unsafe { syscall(SYS_DUP2, oldfd as i64, newfd as i64) as i64 }
    }

    /// Sleep for seconds (uses nanosleep internally via libc wrapper)
    pub fn sleep(seconds: u32) -> u32 {
        unsafe { libc::sleep(seconds) }
    }

    /// Sleep for microseconds (uses usleep internally)
    pub fn usleep(useconds: u32) -> i32 {
        unsafe { libc::usleep(useconds) }
    }

    /// Get current time in seconds since epoch
    pub fn time_now() -> i64 {
        unsafe { libc::time(std::ptr::null_mut()) }
    }

    /// Run system command
    pub fn system(cmd_ptr: i64) -> i32 {
        let cmd = unsafe {
            let ptr = cmd_ptr as *const u8;
            std::ffi::CStr::from_ptr(ptr as *const i8)
                .to_string_lossy()
                .into_owned()
        };
        let c_cmd = std::ffi::CString::new(cmd).unwrap();
        unsafe { libc::system(c_cmd.as_ptr()) }
    }

    /// Get environment variable
    pub fn getenv(name_ptr: i64) -> i64 {
        let name = unsafe {
            let ptr = name_ptr as *const u8;
            std::ffi::CStr::from_ptr(ptr as *const i8)
                .to_string_lossy()
                .into_owned()
        };
        let c_name = std::ffi::CString::new(name).unwrap();
        let result = unsafe { libc::getenv(c_name.as_ptr()) };
        result as i64
    }

    /// Get current working directory
    pub fn getcwd(buf_ptr: i64, size: usize) -> i64 {
        let buf = unsafe { std::slice::from_raw_parts_mut(buf_ptr as *mut u8, size) };
        match unsafe { libc::getcwd(buf.as_mut_ptr() as *mut i8, size) } {
            ptr if ptr.is_null() => -1,
            _ => buf_ptr,
        }
    }

    /// Change current directory
    pub fn chdir(path_ptr: i64) -> i64 {
        let path = unsafe {
            let ptr = path_ptr as *const u8;
            std::ffi::CStr::from_ptr(ptr as *const i8)
                .to_string_lossy()
                .into_owned()
        };
        let c_path = std::ffi::CString::new(path).unwrap();
        unsafe { libc::chdir(c_path.as_ptr()) as i64 }
    }
}

#[cfg(not(target_os = "linux"))]
pub mod os {
    pub fn fork() -> i64 { -1 }
    pub fn exit(_c: i32) -> i64 { -1 }
    pub fn wait(_p: i64) -> i64 { -1 }
    pub fn getpid() -> i64 { -1 }
    pub fn kill(_p: i64, _s: i32) -> i64 { -1 }
    pub fn open(_p: i64, _f: i32) -> i64 { -1 }
    pub fn close(_f: i32) -> i64 { -1 }
    pub fn read(_f: i32, _b: i64, _c: usize) -> i64 { -1 }
    pub fn write(_f: i32, _b: i64, _c: usize) -> i64 { -1 }
    pub fn pipe(_r: i64, _w: i64) -> i64 { -1 }
    pub fn dup(_o: i32) -> i64 { -1 }
    pub fn dup2(_o: i32, _n: i32) -> i64 { -1 }
    pub fn sleep(_s: u32) -> u32 { 0 }
    pub fn usleep(_u: u32) -> i32 { -1 }
    pub fn time_now() -> i64 { -1 }
    pub fn system(_c: i64) -> i32 { -1 }
    pub fn getenv(_n: i64) -> i64 { -1 }
    pub fn getcwd(_b: i64, _s: usize) -> i64 { -1 }
    pub fn chdir(_p: i64) -> i64 { -1 }
}
