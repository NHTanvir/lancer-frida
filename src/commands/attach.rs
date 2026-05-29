use crate::DeviceArgs;
use crate::device::device_flags;
use crate::script::load_source;
use std::path::Path;
use std::process::Command;

pub fn run(args: &DeviceArgs, target: &str, script_file: Option<&Path>, eval: Option<&str>) -> anyhow::Result<()> {
    let source = load_source(script_file, eval)?;
    // Write source to temp file and pass via -l, or pass inline via -e (frida arg limit)
    let mut cmd = Command::new("frida");
    cmd.args(device_flags(args)).arg(target).arg("-e").arg(&source);
    let status = cmd.status()?;
    if !status.success() { anyhow::bail!("frida exited with {}", status); }
    Ok(())
}
