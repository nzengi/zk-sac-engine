use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src/zkvm/programs/guest_program.rs");
    
    // Check if we're building with risc0 feature
    if env::var("CARGO_FEATURE_RISC0").is_ok() {
        build_guest_program();
    }
}

fn build_guest_program() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    // For now, create a simple test ELF since risc0-build is complex on MacOS
    let guest_elf_path = out_dir.join("guest-program");
    create_test_elf(&guest_elf_path);
    
    println!("cargo:rustc-env=GUEST_ELF_PATH={}", guest_elf_path.display());
    println!("cargo:warning=Created test guest ELF at {}", guest_elf_path.display());
}

fn create_test_elf(output_path: &PathBuf) {
    // Create a minimal valid RISC-V ELF binary
    let mut elf_data = Vec::new();
    
    // ELF header (32-bit, little-endian, RISC-V)
    elf_data.extend_from_slice(&[
        0x7f, 0x45, 0x4c, 0x46, // ELF magic
        0x01,                    // 32-bit
        0x01,                    // little-endian
        0x01,                    // ELF version
        0x00,                    // OS ABI (System V)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // padding
        0x02, 0x00,              // executable file
        0xf3, 0x00,              // RISC-V architecture
        0x01, 0x00, 0x00, 0x00,  // ELF version
        0x00, 0x00, 0x00, 0x00,  // entry point
        0x34, 0x00, 0x00, 0x00,  // program header offset
        0x00, 0x00, 0x00, 0x00,  // section header offset
        0x00, 0x00, 0x00, 0x00,  // flags
        0x34, 0x00,              // ELF header size
        0x20, 0x00,              // program header entry size
        0x01, 0x00,              // program header count
        0x28, 0x00,              // section header entry size
        0x01, 0x00,              // section header count
        0x00, 0x00,              // section name string table index
    ]);
    
    // Program header
    elf_data.extend_from_slice(&[
        0x01, 0x00, 0x00, 0x00, // loadable segment
        0x05, 0x00, 0x00, 0x00, // read + execute
        0x00, 0x00, 0x00, 0x00, // offset
        0x00, 0x00, 0x00, 0x00, // virtual address
        0x00, 0x00, 0x00, 0x00, // physical address
        0x54, 0x00, 0x00, 0x00, // file size
        0x54, 0x00, 0x00, 0x00, // memory size
        0x00, 0x10, 0x00, 0x00, // alignment
    ]);
    
    // Simple RISC-V code (just return)
    elf_data.extend_from_slice(&[
        0x67, 0x80, 0x00, 0x00, // ret instruction
        0x00, 0x00, 0x00, 0x00, // padding
    ]);
    
    // Pad to 128 bytes
    while elf_data.len() < 128 {
        elf_data.push(0);
    }
    
    std::fs::write(output_path, &elf_data).expect("Failed to write guest ELF");
} 