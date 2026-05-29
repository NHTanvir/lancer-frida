use crate::DeviceArgs;
use crate::device::device_flags;
use std::process::Command;

pub fn run(args: &DeviceArgs) -> anyhow::Result<()> {
    let status = Command::new("frida-ps")
        .args(device_flags(args))
        .status()?;
    if !status.success() {
        anyhow::bail!("frida-ps exited with {}", status);
    }
    Ok(())
}
