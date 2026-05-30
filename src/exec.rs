//! Helpers for delegating to the upstream Frida command-line tools.
//!
//! `lancer-frida` follows the plan's Path A: the instrumentation engine is the
//! real Frida, reached by shelling out to the `frida-*` binaries (installed via
//! `pip install frida-tools`). These helpers centralise process spawning and
//! give a friendly hint when a tool is missing from `PATH`.

use std::ffi::OsStr;
use std::process::Command;

/// Run an upstream tool, inheriting this process's stdio, forwarding `args`.
/// Returns an error (with an install hint) if the binary is missing or it exits
/// non-zero.
pub fn run<I, S>(bin: &str, args: I) -> anyhow::Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let status = Command::new(bin)
        .args(args)
        .status()
        .map_err(|e| missing(bin, e))?;
    if !status.success() {
        anyhow::bail!("{bin} exited with {status}");
    }
    Ok(())
}

/// Run an upstream tool and capture its stdout as a `String`.
pub fn capture<I, S>(bin: &str, args: I) -> anyhow::Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new(bin)
        .args(args)
        .output()
        .map_err(|e| missing(bin, e))?;
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// True if `bin` is resolvable on `PATH` (best-effort: tries `--version`).
pub fn available(bin: &str) -> bool {
    Command::new(bin)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

fn missing(bin: &str, e: std::io::Error) -> anyhow::Error {
    if e.kind() == std::io::ErrorKind::NotFound {
        anyhow::anyhow!(
            "`{bin}` not found on PATH — install the Frida tools with `pip install frida-tools`"
        )
    } else {
        anyhow::anyhow!("failed to launch {bin}: {e}")
    }
}
