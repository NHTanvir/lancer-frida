use crate::{device::get_device, DeviceArgs, FRIDA};
use frida::DeviceManager;

pub fn run(args: &DeviceArgs) -> anyhow::Result<()> {
    let manager = DeviceManager::obtain(&FRIDA);
    let device = get_device(&manager, args)?;

    let processes = device.enumerate_processes();
    if processes.is_empty() {
        println!("No processes found.");
        return Ok(());
    }

    println!("{:>6}  {}", "PID", "Name");
    println!("{:->6}  {:-<40}", "", "");
    for p in &processes {
        println!("{:>6}  {}", p.get_pid(), p.get_name());
    }
    Ok(())
}
