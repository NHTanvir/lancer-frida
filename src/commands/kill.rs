use crate::DeviceArgs;
use crate::device::device_flags;
use std::process::Command;

pub fn run(args: &DeviceArgs, target: &str) -> anyhow::Result<()> {
    let status = Command::new("frida-kill")
        .args(device_flags(args))
        .arg(target)
        .status()?;
    if !status.success() {
        anyhow::bail!("frida-kill exited with {}", status);
    }
    Ok(())
}
