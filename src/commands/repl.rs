use crate::{DeviceArgs, FRIDA};
use crate::device::{get_device, resolve_pid};
use crate::script::ChannelHandler;
use frida::{DeviceManager, ScriptOption};
use std::io::{self, Write};
use std::sync::mpsc;
use std::time::Duration;

const REPL_AGENT: &str = r#"
(function () {
    var _ctx = {};

    function evalInCtx(code) {
        // Allow assignments to persist in _ctx
        return (new Function('_ctx', 'with (_ctx) { return (' + code + '); }'))(_ctx);
    }

    function onEval(msg) {
        if (!msg || msg.type !== 'eval') {
            recv('eval', onEval);
            return;
        }
        try {
            var r = evalInCtx(msg.expression);
            var repr;
            if (r === undefined) {
                repr = 'undefined';
            } else if (r === null) {
                repr = 'null';
            } else {
                try { repr = JSON.stringify(r, null, 2); } catch (_) { repr = String(r); }
            }
            send({ ok: true, result: repr });
        } catch (e) {
            send({ ok: false, error: e.toString() });
        }
        recv('eval', onEval);
    }

    recv('eval', onEval);
    send({ ok: true, result: 'REPL ready — type JS, .exit to quit' });
})();
"#;

pub fn run(device_args: &DeviceArgs, target: &str) -> anyhow::Result<()> {
    let manager = DeviceManager::obtain(&FRIDA);
    let device = get_device(&manager, device_args)?;
    let pid = resolve_pid(&device, target)?;

    println!("[*] Attaching to '{}' (pid {})...", target, pid);
    let session = device.attach(pid)?;

    let (tx, rx) = mpsc::channel();
    let handler = ChannelHandler { tx };
    let mut script_opt = ScriptOption::default();
    let mut script = session.create_script(REPL_AGENT, &mut script_opt)?;
    script.handle_message(handler)?;
    script.load()?;

    // Wait for ready signal
    match rx.recv_timeout(Duration::from_secs(10)) {
        Ok(v) => {
            if let Some(msg) = v["result"].as_str() {
                println!("[*] {}", msg);
            }
        }
        Err(_) => anyhow::bail!("timeout waiting for REPL agent to initialise"),
    }

    println!("[*] Type JavaScript expressions. '.exit' or Ctrl+D to quit.\n");

    loop {
        print!("[frida]> ");
        io::stdout().flush()?;

        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(0) => break, // EOF (Ctrl+D)
            Ok(_) => {}
            Err(e) => return Err(e.into()),
        }

        let expr = line.trim();
        if expr.is_empty() {
            continue;
        }
        if expr == ".exit" || expr == "quit()" || expr == "exit" {
            break;
        }

        let msg = serde_json::to_string(&serde_json::json!({
            "type": "eval",
            "expression": expr
        }))?;

        script.post(&msg, None)?;

        match rx.recv_timeout(Duration::from_secs(30)) {
            Ok(v) => {
                if let Some(false) = v["ok"].as_bool() {
                    eprintln!("! {}", v["error"].as_str().unwrap_or("unknown error"));
                } else if let Some(result) = v["result"].as_str() {
                    println!("{}", result);
                }
            }
            Err(_) => eprintln!("! timeout waiting for script response"),
        }
    }

    let _ = script.unload();
    let _ = session.detach();
    println!("\n[*] Session ended.");
    Ok(())
}
