//! Build script for Shadow-EVM host
//!
//! Compiles the guest program and generates the methods.rs file
//! containing the ELF binary and image ID.

fn main() {
    // Tell Cargo to rerun if guest code changes
    println!("cargo:rerun-if-changed=../guest/src");
    println!("cargo:rerun-if-changed=../guest/Cargo.toml");

    // Build the guest program
    risc0_build::embed_methods();
}
