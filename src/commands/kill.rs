use crate::{
    device::{get_device, resolve_pid},
    DeviceArgs, FRIDA,
};
use frida::DeviceManager;

pub fn run(args: &DeviceArgs, target: &str) -> anyhow::Result<()> {
    let manager = DeviceManager::obtain(&FRIDA);
    let mut device = get_device(&manager, args)?;
    let pid = resolve_pid(&device, target)?;
    device.kill(pid)?;
    println!("Killed '{}' (pid {})", target, pid);
    Ok(())
}
