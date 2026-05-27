use crate::{
    device::{get_device, resolve_pid},
    script::{load_source, LogHandler},
    DeviceArgs, FRIDA,
};
use frida::{DeviceManager, ScriptOption};
use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

pub fn run(
    args: &DeviceArgs,
    target: &str,
    script_file: Option<&Path>,
    eval: Option<&str>,
) -> anyhow::Result<()> {
    let source = load_source(script_file, eval)?;

    let manager = DeviceManager::obtain(&FRIDA);
    let device = get_device(&manager, args)?;
    let pid = resolve_pid(&device, target)?;

    println!("[*] Attaching to '{}' (pid {})...", target, pid);
    let session = device.attach(pid)?;
    if session.is_detached() {
        anyhow::bail!("failed to attach to pid {}", pid);
    }

    let mut opt = ScriptOption::default();
    let mut script = session.create_script(&source, &mut opt)?;
    script.handle_message(LogHandler)?;
    script.load()?;
    println!("[*] Script loaded. Press Ctrl+C to detach.\n");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    while running.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    println!("\n[*] Detaching...");
    let _ = script.unload();
    let _ = session.detach();
    Ok(())
}
