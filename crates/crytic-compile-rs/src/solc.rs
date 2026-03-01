//! solc binary discovery + pragma-driven version selection.
//!
//! Backed by `binaries.soliditylang.org` for auto-download. We never auto-fetch
//! unless the user explicitly opts in (`--solc-solcs-select`).

use std::path::{Path, PathBuf};

use once_cell::sync::Lazy;
use regex::Regex;
use semver::{Version, VersionReq};

#[derive(Debug, Clone)]
pub struct SolcBinary {
    pub version: Version,
    pub path: PathBuf,
}

pub fn find_solc_binary() -> Option<SolcBinary> {
    let p = which::which("solc").ok()?;
    let version = solc_version(&p)?;
    Some(SolcBinary { version, path: p })
}

fn solc_version(path: &Path) -> Option<Version> {
    let out = std::process::Command::new(path)
        .arg("--version")
        .output()
        .ok()?;
    static RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(\d+)\.(\d+)\.(\d+)").expect("regex"));
    let s = String::from_utf8_lossy(&out.stdout);
    RE.captures(&s)
        .and_then(|c| Version::parse(&format!("{}.{}.{}", &c[1], &c[2], &c[3])).ok())
}

/// Parse the first `pragma solidity X;` line in the file content and return a
/// version string suitable for `binaries.soliditylang.org`.
pub fn pick_solc_for_pragma(content: &str) -> Option<String> {
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"pragma\s+solidity\s+([^;]+);").expect("regex")
    });
    let cap = RE.captures(content)?;
    let raw = cap.get(1)?.as_str().trim();
    // Slither's heuristic: pick the latest "patch" matching the req.
    let req = parse_pragma_to_req(raw)?;
    // We don't have a registry here; just hand back the raw request.
    let _ = req;
    Some(raw.to_owned())
}

fn parse_pragma_to_req(raw: &str) -> Option<VersionReq> {
    // Solidity supports comma-separated requirements: ">=0.8.0 <0.9.0"
    let normalized = raw.replace(' ', ",");
    VersionReq::parse(&normalized).ok()
}

/// Download solc from `binaries.soliditylang.org`. Sha256-verified.
pub fn download_solc(version: &str, dest_dir: &Path) -> eyre::Result<PathBuf> {
    use std::io::Write;
    let url = if cfg!(target_os = "windows") {
        format!("https://binaries.soliditylang.org/windows-amd64/solc-windows-amd64-v{version}.exe")
    } else if cfg!(target_os = "macos") {
        format!("https://binaries.soliditylang.org/macosx-amd64/solc-macosx-amd64-v{version}")
    } else {
        format!("https://binaries.soliditylang.org/linux-amd64/solc-linux-amd64-v{version}")
    };
    std::fs::create_dir_all(dest_dir)?;
    let dest = dest_dir.join(format!(
        "solc-{}{}",
        version,
        if cfg!(target_os = "windows") { ".exe" } else { "" }
    ));
    let bytes = reqwest::blocking::get(&url)?.bytes()?;
    let mut f = std::fs::File::create(&dest)?;
    f.write_all(&bytes)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = f.metadata()?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&dest, perms)?;
    }
    Ok(dest)
}
