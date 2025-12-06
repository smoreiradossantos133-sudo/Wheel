use std::process::Command;
use std::fs;

#[test]
fn builds_and_runs_hello_example() {
    // Build wheelc in release
    let status = Command::new("cargo")
        .arg("build").arg("--release")
        .status()
        .expect("failed to spawn cargo build");
    assert!(status.success());

    // Generate executable
    let wc = std::path::Path::new("target/release/wheelc");
    assert!(wc.exists(), "wheelc not built");

    let status2 = Command::new(wc)
        .arg("examples/hello.wheel")
        .arg("-o")
        .arg("hello_test")
        .arg("--mode")
        .arg("ge")
        .status()
        .expect("failed to run wheelc");
    assert!(status2.success());

    // Run generated program and capture output
    let output = Command::new("./hello_test").output().expect("failed to execute hello_test");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello, Wheel world!"), "unexpected output: {}", stdout);
}
