use crate::DeviceArgs;
use crate::device::device_flags;
use std::process::Command;

pub fn run(device_args: &DeviceArgs, target: &str) -> anyhow::Result<()> {
    let status = Command::new("frida").args(device_flags(device_args)).arg(target).status()?;
    if !status.success() { anyhow::bail!("frida exited with {}", status); }
    Ok(())
}
