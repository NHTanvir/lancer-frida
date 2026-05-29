use crate::DeviceArgs;
use crate::device::device_flags;
use crate::script::load_source;
use std::path::Path;
use std::process::Command;

pub fn run(args: &DeviceArgs, program: &str, prog_args: &[String], script_file: Option<&Path>, eval: Option<&str>) -> anyhow::Result<()> {
    let source = load_source(script_file, eval)?;
    let mut cmd = Command::new("frida");
    cmd.args(device_flags(args)).arg("--spawn").arg(program).arg("-e").arg(&source);
    if !prog_args.is_empty() { cmd.arg("--").args(prog_args); }
    let status = cmd.status()?;
    if !status.success() { anyhow::bail!("frida exited with {}", status); }
    Ok(())
}
