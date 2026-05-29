use crate::DeviceArgs;
use crate::device::device_flags;
use std::process::Command;

const ANDROID_SCRIPT: &str = "Java.perform(function(){ Java.enumerateLoadedClassesSync().forEach(function(c){ console.log(c); }); });";
const IOS_SCRIPT: &str = "if(ObjC.available){Object.keys(ObjC.classes).forEach(function(c){console.log(c);})}";

pub fn run(device: &DeviceArgs, target: &str, filter: Option<&str>, platform: &str) -> anyhow::Result<()> {
    let script = if platform == "ios" { IOS_SCRIPT } else { ANDROID_SCRIPT };
    let output = Command::new("frida").args(device_flags(device)).arg(target).arg("-e").arg(script).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(f) = filter {
            if !line.to_lowercase().contains(&f.to_lowercase()) { continue; }
        }
        println!("{}", line);
    }
    Ok(())
}
