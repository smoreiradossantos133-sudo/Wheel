use std::fs::{self, OpenOptions};
use std::io::Write;
use std::process::Command;
use std::path::PathBuf;
use clap::Parser;
use anyhow::{Result, Context};
use std::collections::HashSet;

mod lexer;
mod parser;
mod ast;
mod codegen;
mod elf_writer;
mod imports;
mod llvm_backend;
use parser::Parser as WheelParser;
use codegen::codegen_to_asm;
use imports::process_imports;

/// Target OS and configuration
#[derive(Debug, Clone, Copy)]
enum TargetOS {
    Linux,
    Windows,
    MacOS,
    Unknown,
}

impl TargetOS {
    fn current() -> Self {
        match std::env::consts::OS {
            "linux" => TargetOS::Linux,
            "windows" => TargetOS::Windows,
            "macos" => TargetOS::MacOS,
            _ => TargetOS::Unknown,
        }
    }

    fn triple(&self) -> &'static str {
        match self {
            TargetOS::Linux => "x86_64-unknown-linux-gnu",
            TargetOS::Windows => "x86_64-pc-windows-msvc",
            TargetOS::MacOS => "x86_64-apple-darwin",
            TargetOS::Unknown => "x86_64-unknown-linux-gnu",
        }
    }

    fn linker(&self) -> &'static str {
        match self {
            TargetOS::Linux => "gcc",
            TargetOS::Windows => "lld-link",
            TargetOS::MacOS => "ld64",
            TargetOS::Unknown => "gcc",
        }
    }

    fn executable_extension(&self) -> &'static str {
        match self {
            TargetOS::Linux => "",
            TargetOS::Windows => ".exe",
            TargetOS::MacOS => "",
            TargetOS::Unknown => "",
        }
    }

    fn format_name(&self) -> &'static str {
        match self {
            TargetOS::Linux => "ELF (Linux)",
            TargetOS::Windows => "PE (Windows)",
            TargetOS::MacOS => "Mach-O (macOS)",
            TargetOS::Unknown => "Unknown",
        }
    }
}

/// Wheel compiler (MVP)
#[derive(Parser)]
#[command(author, version, about = "Wheel compiler (MVP) - generates native binaries for x86_64 Linux")]
struct Cli {
    /// Input .wheel source file
    input: PathBuf,

    /// Output file
    #[arg(short = 'o', long = "out", default_value = "a.out")]
    output: PathBuf,

    /// Mode: ge generate executable, gb generate raw binary
    #[arg(long = "mode", default_value = "ge")]
    mode: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let src = fs::read_to_string(&cli.input)
        .with_context(|| format!("failed to read input file {}", cli.input.display()))?;

    // First-run installer: detect install root and add to PATH
    // Marker file: ~/.wheel_installed
    let marker = dirs::home_dir().map(|d| d.join(".wheel_installed"));
    if let Some(marker_path) = marker {
        if !marker_path.exists() {
            // determine install root as the directory containing current executable
            if let Ok(exe) = std::env::current_exe() {
                if let Some(root) = exe.parent() {
                    let root_str = root.to_string_lossy().to_string();
                    eprintln!("First run detected: adding {} to PATH in shell profiles", root_str);
                    let shells = [".profile", ".bashrc", ".zshrc"];
                    if let Some(home) = dirs::home_dir() {
                        for sh in &shells {
                            let p = home.join(sh);
                            if p.exists() {
                                if let Ok(mut s) = fs::read_to_string(&p) {
                                    let export_line = format!("\n# Added by Wheel installer\nexport PATH=\"{}:$PATH\"\n", root_str);
                                    if !s.contains(&export_line) {
                                        if let Ok(mut f) = fs::OpenOptions::new().append(true).open(&p) {
                                            let _ = f.write_all(export_line.as_bytes());
                                        }
                                    }
                                }
                            } else {
                                // create profile file with export
                                let export_line = format!("# Added by Wheel installer\nexport PATH=\"{}:$PATH\"\n", root_str);
                                let _ = fs::write(&p, export_line);
                            }
                        }
                    }
                }
            }
            // create marker to avoid repeating
            if let Some(parent) = marker_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(marker_path, b"installed");
        }
    }

    // Debug: dump lexer tokens if WHEEL_LEX_DUMP env var is set
    if std::env::var("WHEEL_LEX_DUMP").is_ok() {
        use crate::lexer::Lexer;
        let mut lx = Lexer::new(&src);
        loop {
            let t = lx.next_token();
            eprintln!("TOKEN: {:?}", t);
            if t == crate::lexer::Token::EOF { break; }
        }
        return Ok(());
    }

    // parse program and process imports
    let mut p = WheelParser::new(&src);
    let mut prog = p.parse_program();
    
    // Process imports by loading and merging imported files
    let input_dir = cli.input.parent().unwrap_or_else(|| std::path::Path::new("."));
    let mut processed_imports = HashSet::new();
    process_imports(&mut prog, input_dir, &mut processed_imports)?;

    if cli.mode == "ge" {
        // Generate executable using assembly + gcc/clang
        let asm = codegen_to_asm(&prog);
        // normal executable: asm as generated
        let asm_path = std::env::temp_dir().join("wheel_tmp.s");
        fs::write(&asm_path, asm.as_bytes())?;

        let target_os = std::env::consts::OS;
        let status = if target_os == "linux" {
            Command::new("gcc")
                .arg("-nostdlib")
                .arg("-o").arg(&cli.output)
                .arg(&asm_path)
                .status()?
        } else if target_os == "macos" {
            Command::new("clang")
                .arg("-nostdlib")
                .arg("-o").arg(&cli.output)
                .arg(&asm_path)
                .status()?
        } else {
            anyhow::bail!("Unsupported OS: {}", target_os)
        };

        if !status.success() {
            anyhow::bail!("compiler failed");
        }

        println!("Generated executable: {}", cli.output.display());

    } else if cli.mode == "gb" {
        // Generate flat binary using assembly + gcc
        // Prepend a minimal Multiboot header to help bootloaders (GRUB) detect
        // the image. This is a simple header; building a full kernel image may
        // require a proper linker script and more advanced layout.
        let mut asm = String::new();
        asm.push_str("    .section .multiboot\n    .align 4\n    .long 0x1BADB002\n    .long 0x00010003\n    .long -(0x1BADB002 + 0x00010003)\n\n");
        asm.push_str(&codegen_to_asm(&prog));
        let asm_path = std::env::temp_dir().join("wheel_tmp.s");
        fs::write(&asm_path, asm.as_bytes())?;

        let exe = cli.output.with_extension("exe");
        let status = Command::new("gcc")
            .arg("-nostdlib")
            .arg("-o").arg(&exe)
            .arg(&asm_path)
            .status()
            .context("failed to run gcc")?;

        if !status.success() {
            anyhow::bail!("gcc failed");
        }

        let status2 = Command::new("objcopy")
            .arg("-O").arg("binary")
            .arg(&exe)
            .arg(&cli.output)
            .status()
            .context("failed to run objcopy")?;

        if !status2.success() {
            anyhow::bail!("objcopy failed");
        }

        println!("Generated flat binary: {}", cli.output.display());
    } else if cli.mode == "ll" {
        // LLVM backend path (requires building with `--features llvm`)
        #[cfg(feature = "llvm")]
        {
            let target_os = TargetOS::current();
            let target_triple = target_os.triple();
            let linker = target_os.linker();
            
            eprintln!("Target: {} ({})", target_os.format_name(), target_triple);
            
            let out_obj = cli.output.with_extension("o");
            let extra_links = llvm_backend::llvm::compile_with_llvm_target(&prog, &cli.output, target_triple)
                .context("llvm compilation failed")?;

            // compiled object file should be at <output>.o; link with system linker
            let mut cmd = Command::new(linker);
            cmd.arg("-o").arg(&cli.output)
                .arg(&out_obj);
            
            // Add library linking flags based on presence of wrapper objects
            let sdl_obj = std::path::Path::new("src/stdlib/sdl_wrappers.o");
            if sdl_obj.exists() {
                cmd.arg("src/stdlib/sdl_wrappers.o").arg("-lSDL2");
            }

            let os_obj = std::path::Path::new("src/stdlib/os_wrappers.o");
            if os_obj.exists() {
                cmd.arg("src/stdlib/os_wrappers.o");
            }

            let luck_obj = std::path::Path::new("src/stdlib/luck.o");
            if luck_obj.exists() {
                cmd.arg("src/stdlib/luck.o");
            }

            let memory_obj = std::path::Path::new("src/stdlib/memory.o");
            if memory_obj.exists() {
                cmd.arg("src/stdlib/memory.o");
            }

            let hwio_obj = std::path::Path::new("src/stdlib/hwio.o");
            if hwio_obj.exists() {
                cmd.arg("src/stdlib/hwio.o");
            }

            let filesystem_obj = std::path::Path::new("src/stdlib/filesystem.o");
            if filesystem_obj.exists() {
                cmd.arg("src/stdlib/filesystem.o");
            }

            let process_obj = std::path::Path::new("src/stdlib/process.o");
            if process_obj.exists() {
                cmd.arg("src/stdlib/process.o");
            }
            
            // Append any extra link args returned by the LLVM backend (e.g., local lib .o/.so)
            for arg in extra_links {
                cmd.arg(arg);
            }

            let status = cmd.status()
                .context("failed to link object with linker")?;

            if !status.success() {
                anyhow::bail!("linking failed");
            }

            let exe_name = if let Some(file_name) = cli.output.file_name() {
                format!("{}{}", file_name.to_string_lossy(), target_os.executable_extension())
            } else {
                format!("{}{}", cli.output.display(), target_os.executable_extension())
            };
            
            println!("Generated executable (LLVM): {} ({})", exe_name, target_os.format_name());
        }
        #[cfg(not(feature = "llvm"))]
        {
            anyhow::bail!("LLVM backend not enabled. Rebuild with `--features llvm`");
        }
    } else {
        println!("Unknown mode: {}. Use 'ge', 'gb' or 'll'", cli.mode);
    }

    Ok(())
}
