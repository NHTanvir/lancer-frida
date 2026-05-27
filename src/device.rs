use crate::{DeviceArgs, FRIDA};
use frida::{Device, DeviceManager, DeviceType};

pub fn get_device<'a>(manager: &'a DeviceManager, args: &DeviceArgs) -> anyhow::Result<Device<'a>> {
    if args.usb {
        let devices = manager.enumerate_all_devices();
        devices
            .into_iter()
            .find(|d| d.get_type() == DeviceType::Usb)
            .ok_or_else(|| anyhow::anyhow!("no USB device found"))
    } else if let Some(ref host) = args.remote {
        manager
            .get_remote_device(host)
            .ok_or_else(|| anyhow::anyhow!("could not connect to remote device at {}", host))
    } else {
        manager
            .get_local_device()
            .ok_or_else(|| anyhow::anyhow!("local device not found"))
    }
}

pub fn resolve_pid(device: &Device, target: &str) -> anyhow::Result<u32> {
    if let Ok(pid) = target.parse::<u32>() {
        return Ok(pid);
    }
    let processes = device.enumerate_processes();
    processes
        .iter()
        .find(|p| p.get_name() == target)
        .map(|p| p.get_pid())
        .ok_or_else(|| anyhow::anyhow!("process '{}' not found", target))
}
