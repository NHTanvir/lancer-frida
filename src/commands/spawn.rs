use crate::{
    device::get_device,
    script::{load_source, LogHandler},
    DeviceArgs, FRIDA,
};
use frida::{DeviceManager, ScriptOption, SpawnOptions};
use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

pub fn run(
    args: &DeviceArgs,
    program: &str,
    prog_args: &[String],
    script_file: Option<&Path>,
    eval: Option<&str>,
) -> anyhow::Result<()> {
    let source = load_source(script_file, eval)?;

    let manager = DeviceManager::obtain(&FRIDA);
    let mut device = get_device(&manager, args)?;

    let all_args: Vec<&str> = std::iter::once(program)
        .chain(prog_args.iter().map(String::as_str))
        .collect();
    let options = SpawnOptions::new().argv(all_args);

    println!("[*] Spawning '{}'...", program);
    let pid = device.spawn(program, &options)?;
    println!("[*] Spawned pid {}. Attaching...", pid);

    let session = device.attach(pid)?;

    let mut opt = ScriptOption::default();
    let mut script = session.create_script(&source, &mut opt)?;
    script.handle_message(LogHandler)?;
    script.load()?;

    println!("[*] Script loaded. Resuming...");
    device.resume(pid)?;
    println!("[*] Running. Press Ctrl+C to stop.\n");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    while running.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    println!("\n[*] Killing process...");
    let _ = script.unload();
    let _ = session.detach();
    let _ = device.kill(pid);
    Ok(())
}
