use crate::DeviceArgs;
use crate::device::device_flags;
use std::process::Command;

const SECURITY_KEYWORDS: &[&str] = &["ssl","tls","cert","trust","pin","auth","login","password","token","secret","key","crypt","hash","hmac","sign","verify","root","jailbreak","emulator","debug","biometric","fingerprint","http","https","request","network","encrypt","decrypt","cipher","aes","rsa"];

pub fn run(device: &DeviceArgs, target: &str, filter: Option<&str>, _ai: bool, limit: usize) -> anyhow::Result<()> {
    let script = "Process.enumerateModules().forEach(function(m){m.enumerateExports().filter(function(e){return e.type==='function';}).forEach(function(e){console.log(m.name+'!'+e.name);});});";
    let output = Command::new("frida").args(device_flags(device)).arg(target).arg("-e").arg(script).arg("--no-pause").output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut count = 0usize;
    for line in stdout.lines() {
        let lower = line.to_lowercase();
        let matches_filter = filter.map(|f| lower.contains(&f.to_lowercase())).unwrap_or_else(|| SECURITY_KEYWORDS.iter().any(|kw| lower.contains(kw)));
        if !matches_filter { continue; }
        println!("{}", line);
        count += 1;
        if limit > 0 && count >= limit { break; }
    }
    eprintln!("[discover] {} results", count);
    Ok(())
}
