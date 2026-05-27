use crate::{DeviceArgs, FRIDA};
use crate::device::{get_device, resolve_pid};
use crate::script::ChannelHandler;
use frida::{DeviceManager, ScriptOption};
use std::sync::mpsc;

// Security-relevant keyword patterns
const SECURITY_KEYWORDS: &[&str] = &[
    "ssl", "tls", "cert", "trust", "pinning", "pin",
    "auth", "login", "password", "token", "secret", "key", "crypt",
    "hash", "hmac", "sign", "verify",
    "root", "jailbreak", "emulator", "debug",
    "biometric", "fingerprint", "face",
    "http", "https", "request", "network",
    "encrypt", "decrypt", "cipher", "aes", "rsa",
];

const DISCOVER_SCRIPT: &str = r#"
(function () {
    var results = [];
    Process.enumerateModules().forEach(function (mod) {
        mod.enumerateExports().forEach(function (exp) {
            if (exp.type === 'function') {
                results.push({ module: mod.name, name: exp.name, address: exp.address });
            }
        });
    });
    send({ exports: results });
    send(null);
})();
"#;

pub fn run(
    device_args: &DeviceArgs,
    target: &str,
    filter: Option<&str>,
    ai: bool,
    limit: usize,
) -> anyhow::Result<()> {
    let manager = DeviceManager::obtain(&FRIDA);
    let device = get_device(&manager, device_args)?;
    let pid = resolve_pid(&device, target)?;
    let session = device.attach(pid)?;

    let (tx, rx) = mpsc::channel();
    let handler = ChannelHandler { tx };
    let mut script_opt = ScriptOption::default();
    let mut script = session.create_script(DISCOVER_SCRIPT, &mut script_opt)?;
    script.handle_message(handler)?;
    script.load()?;

    let mut all_exports: Vec<(String, String, String)> = Vec::new();

    loop {
        match rx.recv() {
            Ok(v) => {
                if v.is_null() {
                    break;
                }
                if let Some(exports) = v["exports"].as_array() {
                    for e in exports {
                        let module = e["module"].as_str().unwrap_or("").to_string();
                        let name = e["name"].as_str().unwrap_or("").to_string();
                        let addr = e["address"].as_str().unwrap_or("").to_string();
                        all_exports.push((module, name, addr));
                    }
                }
            }
            Err(_) => break,
        }
    }

    eprintln!("[discover] total exports: {}", all_exports.len());

    // Filter by security keywords or user-supplied filter
    let interesting: Vec<&(String, String, String)> = all_exports
        .iter()
        .filter(|(module, name, _)| {
            let combined = format!("{} {}", module, name).to_lowercase();
            if let Some(f) = filter {
                combined.contains(&f.to_lowercase())
            } else {
                SECURITY_KEYWORDS.iter().any(|kw| combined.contains(kw))
            }
        })
        .collect();

    eprintln!("[discover] security-relevant exports: {}", interesting.len());

    if !ai || interesting.is_empty() {
        let display_limit = if limit == 0 { interesting.len() } else { limit.min(interesting.len()) };
        println!("{:<40} {:<30} ADDRESS", "MODULE", "EXPORT");
        println!("{}", "-".repeat(85));
        for (module, name, addr) in &interesting[..display_limit] {
            println!("{:<40} {:<30} {}", module, name, addr);
        }
        if limit > 0 && interesting.len() > limit {
            println!("  ... and {} more (use --limit 0 to show all)", interesting.len() - limit);
        }
        return Ok(());
    }

    // AI ranking
    eprintln!("[discover] sending to Claude for security ranking...");

    let entries: Vec<String> = interesting
        .iter()
        .take(200)
        .map(|(m, n, _)| format!("{}!{}", m, n))
        .collect();
    let list = entries.join("\n");

    let prompt = format!(
        "You are a mobile security researcher. From the following list of native function exports \
found in a running process, identify the TOP {} most likely to be involved in security controls \
(SSL pinning, authentication, encryption, root detection, etc.).\n\
For each one, output one line: MODULE!EXPORT — <reason>\n\
List only:\n{}",
        limit.max(20),
        list
    );

    let ranked = crate::commands::generate::call_claude(&prompt)?;
    println!("{}", ranked);
    Ok(())
}
