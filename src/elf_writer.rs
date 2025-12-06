use std::io::Write;
use std::fs::File;

/// Minimal ELF64 executable writer for x86_64 Linux
pub struct ELFWriter {
    text: Vec<u8>,      // .text section (machine code)
    rodata: Vec<u8>,    // .rodata section (read-only data)
}

impl ELFWriter {
    pub fn new() -> Self {
        Self {
            text: Vec::new(),
            rodata: Vec::new(),
        }
    }

    pub fn add_text(&mut self, code: Vec<u8>) {
        self.text.extend(code);
    }

    pub fn add_rodata(&mut self, data: Vec<u8>) {
        self.rodata.extend(data);
    }

    pub fn write_elf(&self, path: &str) -> std::io::Result<()> {
        let mut f = File::create(path)?;

        // Single LOAD segment approach:
        // Vaddr: 0x400000 (text)
        // Offset in segment: text starts at 0, rodata at 0x1000 (page-aligned)
        // Memory layout will be:
        // [.text at 0x400000]
        // [padding to 0x401000]
        // [.rodata at 0x401000]

        let text_file_offset = 64 + 56; // 64 byte header + 56 byte program header
        
        // Pad text to page boundary
        let mut text_padded = self.text.clone();
        while text_padded.len() < 0x1000 {
            text_padded.push(0);
        }

        let text_vaddr = 0x400000u64;
        let rodata_vaddr = 0x401000u64;

        // Build ELF header
        let mut elf_header = vec![0u8; 64];
        
        // e_ident (ELF identification)
        elf_header[0..4].copy_from_slice(b"\x7fELF");  // Magic number
        elf_header[4] = 2;   // 64-bit
        elf_header[5] = 1;   // Little endian
        elf_header[6] = 1;   // Version (current)
        elf_header[7] = 0;   // System V ABI
        
        // e_type: EXEC (2)
        write_u16_le(&mut elf_header, 16, 2);
        
        // e_machine: x86-64 (62)
        write_u16_le(&mut elf_header, 18, 62);
        
        // e_version
        write_u32_le(&mut elf_header, 20, 1);
        
        // e_entry
        write_u64_le(&mut elf_header, 24, text_vaddr);
        
        // e_phoff: program header offset
        write_u64_le(&mut elf_header, 32, 64);
        
        // e_shoff: section header offset (none)
        write_u64_le(&mut elf_header, 40, 0);
        
        // e_flags
        write_u32_le(&mut elf_header, 48, 0);
        
        // e_ehsize: ELF header size
        write_u16_le(&mut elf_header, 52, 64);
        
        // e_phentsize: program header entry size
        write_u16_le(&mut elf_header, 54, 56);
        
        // e_phnum: number of program headers (just 1)
        write_u16_le(&mut elf_header, 56, 1);
        
        // e_shentsize, e_shnum, e_shstrndx (no sections)
        write_u16_le(&mut elf_header, 58, 0);
        write_u16_le(&mut elf_header, 60, 0);
        write_u16_le(&mut elf_header, 62, 0);
        
        f.write_all(&elf_header)?;
        
        // Single program header: both .text and .rodata in one segment
        let segment_size = text_padded.len() + self.rodata.len();
        f.write_all(&make_program_header(
            1,                              // p_type: PT_LOAD
            5,                              // p_flags: PF_R | PF_X (read + execute)
            text_file_offset as u64,        // p_offset
            text_vaddr,                     // p_vaddr
            text_vaddr,                     // p_paddr
            segment_size as u64,            // p_filesz
            segment_size as u64,            // p_memsz
            0x1000,                         // p_align (page-aligned)
        ))?;
        
        // Write segments
        f.write_all(&text_padded)?;
        f.write_all(&self.rodata)?;

        // Make executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o755);
            std::fs::set_permissions(path, perms)?;
        }

        Ok(())
    }
}

fn make_program_header(
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
) -> Vec<u8> {
    let mut phdr = vec![0u8; 56];
    write_u32_le(&mut phdr, 0, p_type);
    write_u32_le(&mut phdr, 4, p_flags);
    write_u64_le(&mut phdr, 8, p_offset);
    write_u64_le(&mut phdr, 16, p_vaddr);
    write_u64_le(&mut phdr, 24, p_paddr);
    write_u64_le(&mut phdr, 32, p_filesz);
    write_u64_le(&mut phdr, 40, p_memsz);
    write_u64_le(&mut phdr, 48, p_align);
    phdr
}

fn write_u16_le(buf: &mut [u8], offset: usize, val: u16) {
    buf[offset..offset+2].copy_from_slice(&val.to_le_bytes());
}

fn write_u32_le(buf: &mut [u8], offset: usize, val: u32) {
    buf[offset..offset+4].copy_from_slice(&val.to_le_bytes());
}

fn write_u64_le(buf: &mut [u8], offset: usize, val: u64) {
    buf[offset..offset+8].copy_from_slice(&val.to_le_bytes());
}

