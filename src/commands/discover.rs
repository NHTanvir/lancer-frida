use crate::DeviceArgs;
use crate::device::device_flags;
use std::process::Command;

const SECURITY_KEYWORDS: &[&str] = &["ssl","tls","cert","trust","pin","auth","login","password","token","secret","key","crypt","hash","hmac","sign","verify","root","jailbreak","emulator","debug","biometric","fingerprint","http","https","request","network","encrypt","decrypt","cipher","aes","rsa"];

const RANK_SYSTEM: &str = r#"You are a mobile reverse-engineering expert triaging an exported-symbol list from a target process.
Rank the supplied `module!export` symbols by how security-interesting they are for a bug-bounty engagement
(authentication, crypto, certificate pinning, root/jailbreak/anti-debug detection, secret handling, network).
Return a concise ranked list, most interesting first, each line formatted `module!export — why it matters (<=10 words)`.
Output ONLY the ranked list, no preamble."#;

pub fn run(device: &DeviceArgs, target: &str, filter: Option<&str>, ai: bool, limit: usize) -> anyhow::Result<()> {
    let script = "Process.enumerateModules().forEach(function(m){m.enumerateExports().filter(function(e){return e.type==='function';}).forEach(function(e){console.log(m.name+'!'+e.name);});});";
    let output = Command::new("frida").args(device_flags(device)).arg(target).arg("-e").arg(script).arg("--no-pause").output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut hits: Vec<String> = Vec::new();
    for line in stdout.lines() {
        let lower = line.to_lowercase();
        let matches_filter = filter
            .map(|f| lower.contains(&f.to_lowercase()))
            .unwrap_or_else(|| SECURITY_KEYWORDS.iter().any(|kw| lower.contains(kw)));
        if !matches_filter { continue; }
        hits.push(line.to_string());
        if limit > 0 && hits.len() >= limit { break; }
    }

    if ai {
        if hits.is_empty() {
            eprintln!("[discover] nothing to rank");
            return Ok(());
        }
        eprintln!("[discover] asking Claude to rank {} symbols by security relevance...", hits.len());
        let prompt = format!("Target: {}\nSymbols:\n{}", target, hits.join("\n"));
        let system = serde_json::json!([{
            "type": "text",
            "text": RANK_SYSTEM,
            "cache_control": { "type": "ephemeral" }
        }]);
        match super::generate::call_claude_with_system(&prompt, Some(system)) {
            Ok(ranked) if !ranked.trim().is_empty() => {
                println!("{}", ranked);
                eprintln!("[discover] {} symbols ranked by AI", hits.len());
                return Ok(());
            }
            Ok(_) => eprintln!("[discover] AI returned nothing — falling back to raw list"),
            Err(e) => eprintln!("[discover] AI ranking failed ({e}) — falling back to raw list"),
        }
    }

    for line in &hits {
        println!("{}", line);
    }
    eprintln!("[discover] {} results", hits.len());
    Ok(())
}
