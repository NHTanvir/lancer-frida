//! Typed Session / Script API (delegated to the `frida` CLI).
//!
//! A `Session` binds a device selector to a target process. `run_script`
//! executes a one-shot agent script and captures its console output — the
//! primitive the GUI console, memory viewer and class browser build on.

use crate::exec;

/// A controller session bound to a device + target.
#[derive(Debug, Clone)]
pub struct Session {
    /// Device selector flags, e.g. `["-U"]`, `["-D", "emulator-5554"]`, `["-H", "host:port"]`.
    pub selector: Vec<String>,
    /// Process name or PID.
    pub target: String,
}

impl Session {
    pub fn new(selector: Vec<String>, target: impl Into<String>) -> Self {
        Session { selector, target: target.into() }
    }

    fn base_args(&self) -> Vec<String> {
        let mut args = self.selector.clone();
        args.push(self.target.clone());
        args
    }

    /// Run a one-shot agent script and capture its console output.
    pub fn run_script(&self, source: &str) -> anyhow::Result<String> {
        let mut args = self.base_args();
        args.push("-e".to_string());
        args.push(source.to_string());
        args.push("--no-pause".to_string());
        exec::capture("frida", args)
    }

    /// Enumerate loaded modules as raw `name\tbase\tsize\tpath` lines.
    pub fn enumerate_modules(&self) -> anyhow::Result<String> {
        self.run_script(
            "Process.enumerateModules().forEach(function(m){console.log(m.name+'\\t'+m.base+'\\t'+m.size+'\\t'+m.path);});",
        )
    }

    /// Enumerate loaded Java (Android) or ObjC (iOS) classes.
    pub fn enumerate_classes(&self, ios: bool) -> anyhow::Result<String> {
        let script = if ios {
            "if(ObjC.available){Object.keys(ObjC.classes).forEach(function(c){console.log(c);})}"
        } else {
            "Java.perform(function(){ Java.enumerateLoadedClassesSync().forEach(function(c){ console.log(c); }); });"
        };
        self.run_script(script)
    }

    /// Read `length` bytes at `address` (hex), returned as a hex string.
    pub fn read_memory(&self, address: &str, length: usize) -> anyhow::Result<String> {
        let script = format!(
            "var b=Memory.readByteArray(ptr('{address}'),{length});console.log(Array.from(new Uint8Array(b)).map(function(x){{return('0'+x.toString(16)).slice(-2)}}).join(' '));"
        );
        self.run_script(&script)
    }
}
