use crate::DeviceArgs;
use crate::device::device_flags;
use std::process::Command;

pub fn modules(device_args: &DeviceArgs, target: &str) -> anyhow::Result<()> {
    let script = "Process.enumerateModules().forEach(function(m){console.log(m.name+'\\t'+m.base+'\\t'+m.size+'\\t'+m.path);});";
    let output = Command::new("frida").args(device_flags(device_args)).arg(target).arg("-e").arg(script).arg("--no-pause").output()?;
    println!("{:<40} {:<18} {:<10} PATH", "NAME", "BASE", "SIZE");
    println!("{}", "-".repeat(90));
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let parts: Vec<&str> = line.splitn(4, '\t').collect();
        if parts.len() == 4 { println!("{:<40} {:<18} {:<10} {}", parts[0], parts[1], parts[2], parts[3]); }
    }
    Ok(())
}

pub fn read(device_args: &DeviceArgs, target: &str, address: &str, length: usize) -> anyhow::Result<()> {
    let script = format!("var b=Memory.readByteArray(ptr('{}'),{});console.log(Array.from(new Uint8Array(b)).map(function(x){{return('0'+x.toString(16)).slice(-2)}}).join(' '));", address, length);
    let output = Command::new("frida").args(device_flags(device_args)).arg(target).arg("-e").arg(&script).arg("--no-pause").output()?;
    print!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

pub fn scan(device_args: &DeviceArgs, target: &str, pattern: &str) -> anyhow::Result<()> {
    let script = format!("Process.enumerateRanges('r--').forEach(function(r){{try{{Memory.scanSync(r.base,r.size,'{}').forEach(function(m){{console.log(m.address+'\\t'+m.size)}})}}catch(e){{}}}});", pattern.replace('\'', "\\'"));
    let output = Command::new("frida").args(device_flags(device_args)).arg(target).arg("-e").arg(&script).arg("--no-pause").output()?;
    print!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}
