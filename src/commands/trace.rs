use crate::{
    device::{get_device, resolve_pid},
    script::LogHandler,
    DeviceArgs, FRIDA,
};
use frida::{DeviceManager, ScriptOption};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

pub fn run(args: &DeviceArgs, target: &str, includes: &[String]) -> anyhow::Result<()> {
    if includes.is_empty() {
        anyhow::bail!("specify at least one pattern with -i, e.g. -i 'open' or -i '*ssl*'");
    }

    let source = build_trace_script(includes);

    let manager = DeviceManager::obtain(&FRIDA);
    let device = get_device(&manager, args)?;
    let pid = resolve_pid(&device, target)?;

    println!("[*] Tracing '{}' (pid {}) — patterns: {:?}", target, pid, includes);
    let session = device.attach(pid)?;
    if session.is_detached() {
        anyhow::bail!("failed to attach to pid {}", pid);
    }

    let mut opt = ScriptOption::default();
    let mut script = session.create_script(&source, &mut opt)?;
    script.handle_message(LogHandler)?;
    script.load()?;
    println!("[*] Interceptors installed. Press Ctrl+C to stop.\n");

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

fn build_trace_script(includes: &[String]) -> String {
    // Encode patterns as a JS array literal
    let pattern_array: String = includes
        .iter()
        .map(|p| format!("\"{}\"", p.replace('\\', "\\\\").replace('"', "\\\"")))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        r#"
(function() {{
    var PATTERNS = [{patterns}];

    function globMatch(pattern, str) {{
        var re = new RegExp(
            '^' + pattern.replace(/[.+^${{}}()|[\]\\\\]/g, '\\\\$&')
                         .replace(/\*/g, '.*')
                         .replace(/\?/g, '.') + '$',
            'i'
        );
        return re.test(str);
    }}

    function shouldTrace(modName, fnName) {{
        if (PATTERNS.length === 0) return true;
        return PATTERNS.some(function(p) {{
            var modPat = '*', fnPat = p;
            if (p.indexOf('!') !== -1) {{
                var parts = p.split('!');
                modPat = parts[0];
                fnPat = parts[1];
            }}
            return globMatch(modPat, modName) && globMatch(fnPat, fnName);
        }});
    }}

    var hooked = 0;
    Process.enumerateModules().forEach(function(mod) {{
        mod.enumerateExports().forEach(function(exp) {{
            if (exp.type !== 'function') return;
            if (!shouldTrace(mod.name, exp.name)) return;
            try {{
                var label = mod.name + '!' + exp.name;
                Interceptor.attach(exp.address, {{
                    onEnter: function(args) {{
                        console.log('[+] ' + label + '()');
                    }},
                    onLeave: function(retval) {{
                        console.log('    <- ' + retval);
                    }}
                }});
                hooked++;
            }} catch(e) {{}}
        }});
    }});

    console.log('[lancer-frida] trace: ' + hooked + ' function(s) hooked');
}})();
"#,
        patterns = pattern_array
    )
}
