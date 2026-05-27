use crate::{DeviceArgs, FRIDA};
use crate::device::{get_device, resolve_pid};
use crate::script::ChannelHandler;
use frida::{DeviceManager, ScriptOption};
use std::sync::mpsc;

pub fn list(
    device_args: &DeviceArgs,
    target: &str,
    script_file: &std::path::Path,
) -> anyhow::Result<()> {
    let source = std::fs::read_to_string(script_file)?;
    let manager = DeviceManager::obtain(&FRIDA);
    let device = get_device(&manager, device_args)?;
    let pid = resolve_pid(&device, target)?;
    let session = device.attach(pid)?;

    let mut script_opt = ScriptOption::default();
    // Use a dummy handler — we only need list_exports
    let (tx, _rx) = mpsc::channel::<serde_json::Value>();
    let handler = ChannelHandler { tx };
    let mut script = session.create_script(&source, &mut script_opt)?;
    script.handle_message(handler)?;
    script.load()?;

    let exports = script.list_exports()?;
    if exports.is_empty() {
        println!("(no rpc.exports found in script)");
    } else {
        println!("rpc.exports:");
        for name in &exports {
            println!("  {}", name);
        }
    }
    let _ = script.unload();
    Ok(())
}

pub fn call(
    device_args: &DeviceArgs,
    target: &str,
    script_file: Option<&std::path::Path>,
    function: &str,
    args: &[String],
) -> anyhow::Result<()> {
    let user_source = crate::script::load_source(script_file, None)?;

    let json_args = args
        .iter()
        .map(|a| serde_json::from_str::<serde_json::Value>(a)
            .unwrap_or_else(|_| serde_json::Value::String(a.clone())))
        .collect::<Vec<_>>();
    let args_json = serde_json::to_string(&json_args)?;

    let source = format!(
        r#"
{}
(function() {{
    if (typeof rpc === 'undefined' || typeof rpc.exports === 'undefined') {{
        send({{ error: 'script does not export rpc.exports' }});
        send(null);
        return;
    }}
    var fn = rpc.exports["{}"];
    if (typeof fn !== 'function') {{
        send({{ error: 'rpc.exports.{} is not a function' }});
        send(null);
        return;
    }}
    Promise.resolve(fn.apply(null, {})).then(function(v) {{
        send({{ result: v }});
        send(null);
    }}).catch(function(e) {{
        send({{ error: String(e) }});
        send(null);
    }});
}})();
"#,
        user_source, function, function, args_json
    );

    let manager = DeviceManager::obtain(&FRIDA);
    let device = get_device(&manager, device_args)?;
    let pid = resolve_pid(&device, target)?;
    let session = device.attach(pid)?;

    let (tx, rx) = mpsc::channel();
    let handler = ChannelHandler { tx };
    let mut script_opt = ScriptOption::default();
    let mut script = session.create_script(&source, &mut script_opt)?;
    script.handle_message(handler)?;
    script.load()?;

    loop {
        match rx.recv() {
            Ok(v) => {
                if v.is_null() { break; }
                if let Some(err) = v.get("error") {
                    anyhow::bail!("rpc error: {}", err);
                }
                if let Some(result) = v.get("result") {
                    println!("{}", serde_json::to_string_pretty(result)?);
                }
            }
            Err(_) => break,
        }
    }

    Ok(())
}
