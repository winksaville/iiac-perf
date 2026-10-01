//! Bakes the build's inputs into the binary for the record: the compiler's version for
//! `host.rustc`, and for `binary` the git commit the tree was at, whether a tracked file differed
//! from it, the profile, its opt-level, and the rustflags cargo passed. Cargo does not put the
//! compiler's version in the build environment, and the rest is what tells two binaries a version
//! string cannot tell apart, so a record exists without anyone looking these up later.

use std::process::Command;

/// A command's trimmed stdout, `None` when it fails to run or exits nonzero.
fn output(cmd: &str, args: &[&str]) -> Option<String> {
    let dir = std::env::var("CARGO_MANIFEST_DIR").ok()?;
    let out = Command::new(cmd)
        .args(args)
        .current_dir(dir)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn main() {
    // The commit and the dirty flag follow the tree, so the script reruns when a tracked input or
    // the checked-out commit changes, not only when it changes itself.
    for path in [
        "build.rs",
        "src",
        "Cargo.toml",
        "Cargo.lock",
        "iiac-perf.example.md",
        "assets",
        ".git/HEAD",
    ] {
        println!("cargo:rerun-if-changed={path}");
    }
    println!("cargo:rerun-if-env-changed=RUSTC");
    println!("cargo:rerun-if-env-changed=RUSTFLAGS");
    println!("cargo:rerun-if-env-changed=CARGO_ENCODED_RUSTFLAGS");

    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let version = match Command::new(rustc).arg("--version").output() {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        _ => "unknown".to_string(),
    };
    println!("cargo:rustc-env=IIAC_PERF_RUSTC={version}");

    let commit = output("git", &["rev-parse", "HEAD"]).unwrap_or_else(|| "unknown".to_string());
    // A tracked file differing from the commit, untracked files aside, since a scratch file is
    // not an input. Unknown outside a repository, which reads as clean.
    let dirty = output("git", &["status", "--porcelain", "--untracked-files=no"])
        .is_some_and(|s| !s.is_empty());
    println!("cargo:rustc-env=IIAC_PERF_COMMIT={commit}");
    println!("cargo:rustc-env=IIAC_PERF_DIRTY={dirty}");
    let env = |k: &str| std::env::var(k).unwrap_or_default();
    println!("cargo:rustc-env=IIAC_PERF_PROFILE={}", env("PROFILE"));
    println!("cargo:rustc-env=IIAC_PERF_OPT_LEVEL={}", env("OPT_LEVEL"));
    // Cargo separates the flags with 0x1f, which a rustc-env value cannot carry.
    let flags = env("CARGO_ENCODED_RUSTFLAGS").replace('\x1f', " ");
    println!("cargo:rustc-env=IIAC_PERF_RUSTFLAGS={flags}");
}
