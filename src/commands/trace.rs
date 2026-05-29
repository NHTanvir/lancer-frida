use crate::DeviceArgs;
use crate::device::device_flags;
use std::process::Command;

pub fn run(args: &DeviceArgs, target: &str, includes: &[String]) -> anyhow::Result<()> {
    if includes.is_empty() { anyhow::bail!("specify at least one pattern with -i"); }
    let mut cmd = Command::new("frida-trace");
    cmd.args(device_flags(args)).arg(target);
    for pat in includes { cmd.arg("-i").arg(pat); }
    let status = cmd.status()?;
    if !status.success() { anyhow::bail!("frida-trace exited with {}", status); }
    Ok(())
}
