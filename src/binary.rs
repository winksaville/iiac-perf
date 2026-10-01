//! The binary a run measured with: its bytes' hash and the inputs that built it.
//!
//! An edit off the hot path moved a bench 7.6% by changing what the compiler inlined, and two
//! builds stamped one version read that far apart, so a version string does not name the build
//! that measured. The hash names it, `sha256sum` of the installed file gives the same hex, and the
//! inputs say why two hashes differ: the commit and whether the tree was dirty, the profile, its
//! opt-level, and the rustflags cargo passed. A profile's own settings in `Cargo.toml` are the
//! commit's.

use std::io::Read;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A binary by its bytes and its build's inputs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Binary {
    /// SHA-256 of the executable's bytes, hex.
    pub sha256: String,
    /// The git commit the build's tree was at, `unknown` outside a repository.
    pub commit: String,
    /// A tracked file differed from `commit` when it was built.
    pub dirty: bool,
    /// Cargo's profile, `release` or `debug`.
    pub profile: String,
    /// The profile's opt-level.
    pub opt_level: String,
    /// The rustflags cargo passed, space-joined, every config file's and `RUSTFLAGS` together.
    pub rustflags: String,
}

impl Binary {
    /// This process's binary: its bytes hashed through `/proc/self/exe`, which names the running
    /// file even after an install has replaced it, and the inputs `build.rs` baked in.
    pub fn this() -> Result<Binary, String> {
        Ok(Binary {
            sha256: sha256_file("/proc/self/exe")?,
            commit: env!("IIAC_PERF_COMMIT").to_string(),
            dirty: env!("IIAC_PERF_DIRTY") == "true",
            profile: env!("IIAC_PERF_PROFILE").to_string(),
            opt_level: env!("IIAC_PERF_OPT_LEVEL").to_string(),
            rustflags: env!("IIAC_PERF_RUSTFLAGS").to_string(),
        })
    }

    /// The hash's first 16 hex digits, the name the banner and `analyze` use.
    pub fn short(&self) -> &str {
        &self.sha256[..self.sha256.len().min(16)]
    }

    /// The banner's line: the short hash, the commit's first 12 digits and whether it was dirty,
    /// the profile and opt-level, and the rustflags when there are any.
    pub fn summary(&self) -> String {
        let commit = &self.commit[..self.commit.len().min(12)];
        let dirty = if self.dirty { " dirty" } else { "" };
        let flags = match self.rustflags.trim() {
            "" => String::new(),
            f => format!(", rustflags {f}"),
        };
        format!(
            "{} ({commit}{dirty}, {} opt {}{flags})",
            self.short(),
            self.profile,
            self.opt_level
        )
    }
}

/// SHA-256 of a file's bytes, hex.
fn sha256_file(path: &str) -> Result<String, String> {
    let mut file = std::fs::File::open(path).map_err(|e| format!("opening {path}: {e}"))?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| format!("reading {path}: {e}"))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hash_is_what_sha256sum_prints() {
        let dir = std::env::temp_dir().join(format!("iiac-binary-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("abc");
        std::fs::write(&file, b"abc").unwrap();
        // The FIPS 180-2 test vector for "abc".
        assert_eq!(
            sha256_file(file.to_str().unwrap()).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn this_binary_names_itself() {
        let b = Binary::this().unwrap();
        assert_eq!(b.sha256.len(), 64);
        assert_eq!(b.short().len(), 16);
        assert!(b.summary().starts_with(b.short()), "{}", b.summary());
        assert!(!b.profile.is_empty());
    }
}
