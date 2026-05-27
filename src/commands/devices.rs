use crate::FRIDA;
use frida::DeviceManager;

pub fn run() -> anyhow::Result<()> {
    let manager = DeviceManager::obtain(&FRIDA);
    let devices = manager.enumerate_all_devices();

    if devices.is_empty() {
        println!("No devices found.");
        return Ok(());
    }

    println!("{:<24} {:<10} {}", "Id", "Type", "Name");
    println!("{}", "-".repeat(60));
    for d in &devices {
        println!("{:<24} {:<10} {}", d.get_id(), format!("{:?}", d.get_type()), d.get_name());
    }
    Ok(())
}
