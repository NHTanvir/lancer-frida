use crate::{DeviceArgs, FRIDA};
use crate::device::{get_device, resolve_pid};
use crate::script::ChannelHandler;
use frida::{DeviceManager, ScriptOption};
use std::sync::mpsc;

const ANDROID_SCRIPT: &str = r#"
(function() {
    Java.perform(function() {
        var classes = Java.enumerateLoadedClassesSync();
        classes.forEach(function(c) { send(c); });
        send(null);
    });
})();
"#;

const OBJC_SCRIPT: &str = r#"
(function() {
    if (ObjC.available) {
        Object.keys(ObjC.classes).forEach(function(c) { send(c); });
    } else {
        send("[classes] ObjC runtime not available");
    }
    send(null);
})();
"#;

pub fn run(device: &DeviceArgs, target: &str, filter: Option<&str>, platform: &str) -> anyhow::Result<()> {
    let manager = DeviceManager::obtain(&FRIDA);
    let device = get_device(&manager, device)?;
    let pid = resolve_pid(&device, target)?;
    let session = device.attach(pid)?;

    let source = if platform == "ios" { OBJC_SCRIPT } else { ANDROID_SCRIPT };

    let (tx, rx) = mpsc::channel();
    let handler = ChannelHandler { tx };

    let mut script_opt = ScriptOption::default();
    let mut script = session.create_script(source, &mut script_opt)?;
    script.handle_message(handler)?;
    script.load()?;

    let mut count = 0usize;
    loop {
        match rx.recv() {
            Ok(val) => {
                if val.is_null() {
                    break;
                }
                let name = val.as_str().unwrap_or_default();
                if let Some(f) = filter {
                    if !name.to_lowercase().contains(&f.to_lowercase()) {
                        continue;
                    }
                }
                println!("{}", name);
                count += 1;
            }
            Err(_) => break,
        }
    }

    eprintln!("[classes] {} classes listed", count);
    Ok(())
}
