//! Bakes the compiler's version into the binary for the record's `host.rustc` field: cargo
//! does not put it in the build environment, and a record exists so nobody has to look it up
//! later.

use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=RUSTC");
    let rustc = match std::env::var("RUSTC") {
        Ok(path) => path,
        Err(_) => "rustc".to_string(),
    };
    let version = match Command::new(rustc).arg("--version").output() {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        _ => "unknown".to_string(),
    };
    println!("cargo:rustc-env=IIAC_PERF_RUSTC={version}");
}
