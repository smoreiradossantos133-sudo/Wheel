// OS helpers and syscall wrappers (skeleton)

pub fn exit(code: i32) -> ! {
    std::process::exit(code)
}

pub fn info() {
    println!("Wheel std::os - placeholder (pid={})", std::process::id());
}

pub fn read_line() -> String {
    use std::io::{self, Write};
    let mut s = String::new();
    io::stdout().flush().ok();
    io::stdin().read_line(&mut s).ok();
    if s.ends_with('\n') { s.pop(); }
    s
}
