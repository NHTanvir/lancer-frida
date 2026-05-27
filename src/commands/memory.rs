use crate::{DeviceArgs, FRIDA};
use crate::device::{get_device, resolve_pid};
use crate::script::ChannelHandler;
use frida::{DeviceManager, ScriptOption};
use std::sync::mpsc;

pub fn modules(device_args: &DeviceArgs, target: &str) -> anyhow::Result<()> {
    let source = r#"
(function() {
    Process.enumerateModules().forEach(function(m) {
        send({ name: m.name, base: m.base, size: m.size, path: m.path });
    });
    send(null);
})();
"#;
    let results = collect(device_args, target, source)?;
    println!("{:<40} {:<18} {:<10} PATH", "NAME", "BASE", "SIZE");
    println!("{}", "-".repeat(90));
    for v in results {
        if v.is_null() { continue; }
        println!(
            "{:<40} {:<18} {:<10} {}",
            v["name"].as_str().unwrap_or(""),
            v["base"].as_str().unwrap_or(""),
            v["size"].as_u64().unwrap_or(0),
            v["path"].as_str().unwrap_or(""),
        );
    }
    Ok(())
}

pub fn read(device_args: &DeviceArgs, target: &str, address: &str, length: usize) -> anyhow::Result<()> {
    let source = format!(
        r#"(function() {{
    var addr = ptr("{}");
    var bytes = Memory.readByteArray(addr, {});
    send({{ hex: Array.from(new Uint8Array(bytes)).map(function(b) {{ return ('0' + b.toString(16)).slice(-2); }}).join('') }});
    send(null);
}})();"#,
        address, length
    );
    let results = collect(device_args, target, &source)?;
    let base_addr = usize::from_str_radix(address.trim_start_matches("0x"), 16).unwrap_or(0);
    for v in results {
        if v.is_null() { continue; }
        if let Some(hex) = v["hex"].as_str() {
            let bytes: Vec<&str> = (0..hex.len()).step_by(2)
                .map(|i| &hex[i..i+2])
                .collect();
            for (i, chunk) in bytes.chunks(16).enumerate() {
                let offset = i * 16;
                let hex_part: Vec<String> = chunk.iter().map(|b| b.to_string()).collect();
                let ascii_part: String = chunk.iter().map(|b| {
                    let byte = u8::from_str_radix(b, 16).unwrap_or(0);
                    if byte >= 0x20 && byte < 0x7f { byte as char } else { '.' }
                }).collect();
                println!("{:#010x}  {:<48}  |{}|",
                    base_addr + offset,
                    hex_part.join(" "),
                    ascii_part
                );
            }
        }
    }
    Ok(())
}

pub fn scan(device_args: &DeviceArgs, target: &str, pattern: &str) -> anyhow::Result<()> {
    let source = format!(
        r#"(function() {{
    var ranges = Process.enumerateRanges('r--');
    ranges.forEach(function(range) {{
        try {{
            Memory.scanSync(range.base, range.size, "{}").forEach(function(match) {{
                send({{ address: match.address, size: match.size }});
            }});
        }} catch(e) {{}}
    }});
    send(null);
}})();"#,
        pattern.replace('"', "\\\"")
    );
    let results = collect(device_args, target, &source)?;
    let mut count = 0usize;
    for v in results {
        if v.is_null() { continue; }
        println!("address={} size={}", v["address"].as_str().unwrap_or("?"), v["size"].as_u64().unwrap_or(0));
        count += 1;
    }
    eprintln!("[memory scan] {} matches", count);
    Ok(())
}

fn collect(device_args: &DeviceArgs, target: &str, source: &str) -> anyhow::Result<Vec<serde_json::Value>> {
    let manager = DeviceManager::obtain(&FRIDA);
    let device = get_device(&manager, device_args)?;
    let pid = resolve_pid(&device, target)?;
    let session = device.attach(pid)?;

    let (tx, rx) = mpsc::channel();
    let handler = ChannelHandler { tx };
    let mut script_opt = ScriptOption::default();
    let mut script = session.create_script(source, &mut script_opt)?;
    script.handle_message(handler)?;
    script.load()?;

    let mut out = Vec::new();
    loop {
        match rx.recv() {
            Ok(v) => {
                let done = v.is_null();
                out.push(v);
                if done { break; }
            }
            Err(_) => break,
        }
    }
    Ok(out)
}
