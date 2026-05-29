use crate::DeviceArgs;
use crate::device::device_flags;
use std::process::Command;

pub fn list(device_args: &DeviceArgs, target: &str, script_file: &std::path::Path) -> anyhow::Result<()> {
    let source = std::fs::read_to_string(script_file)?;
    let list_script = format!("{}\nconsole.log(JSON.stringify(Object.keys(rpc.exports||{{}})));", source);
    let output = Command::new("frida").args(device_flags(device_args)).arg(target).arg("-e").arg(&list_script).arg("--no-pause").output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.starts_with('[') {
            if let Ok(names) = serde_json::from_str::<Vec<String>>(line) {
                println!("rpc.exports:");
                for name in &names { println!("  {}", name); }
                return Ok(());
            }
        }
    }
    println!("(no rpc.exports found or script error)");
    Ok(())
}

pub fn call(device_args: &DeviceArgs, target: &str, script_file: Option<&std::path::Path>, function: &str, args: &[String]) -> anyhow::Result<()> {
    let user_source = crate::script::load_source(script_file, None)?;
    let json_args = args.iter().map(|a| serde_json::from_str::<serde_json::Value>(a).unwrap_or_else(|_| serde_json::Value::String(a.clone()))).collect::<Vec<_>>();
    let args_json = serde_json::to_string(&json_args)?;
    let full = format!("{}\nPromise.resolve(rpc.exports.{}(...{})).then(function(v){{console.log(JSON.stringify(v,null,2));}}).catch(function(e){{console.error(String(e));}});", user_source, function, args_json);
    let status = Command::new("frida").args(device_flags(device_args)).arg(target).arg("-e").arg(&full).arg("--no-pause").status()?;
    if !status.success() { anyhow::bail!("frida exited with {}", status); }
    Ok(())
}
