//! Thin pass-through wrappers over the upstream Frida CLI tools.
//!
//! Every argument is forwarded verbatim to the underlying `frida-*` / `gum-graft`
//! binary, so 100% of each tool's upstream flags work through `lancer-frida`
//! without us having to mirror each flag in clap. The companion binaries in
//! `src/bin/` re-dispatch here too.

use crate::exec;

macro_rules! passthrough {
    ($(#[$doc:meta])* $name:ident => $bin:literal) => {
        $(#[$doc])*
        pub fn $name(args: &[String]) -> anyhow::Result<()> {
            exec::run($bin, args)
        }
    };
}

passthrough!(/// `frida-ls-devices` — enumerate devices/backends.
    ls_devices => "frida-ls-devices");
passthrough!(/// `frida-pull` — copy a file off the target device.
    pull => "frida-pull");
passthrough!(/// `frida-push` — copy a file onto the target device.
    push => "frida-push");
passthrough!(/// `frida-rm` — remove a file on the target.
    rm => "frida-rm");
passthrough!(/// `frida-ls` — list files on the target.
    ls => "frida-ls");
passthrough!(/// `frida-mkdir` — create a directory on the target.
    mkdir => "frida-mkdir");
passthrough!(/// `frida-rmdir` — remove a directory on the target.
    rmdir => "frida-rmdir");
passthrough!(/// `frida-mv` — move/rename a file on the target.
    mv => "frida-mv");
passthrough!(/// `frida-apk` — sign and inject frida-gadget.so into an APK.
    apk => "frida-apk");
passthrough!(/// `frida-create` — scaffold a new agent project.
    create => "frida-create");
passthrough!(/// `frida-compile` — compile a TS/ESM agent into a single bundle.
    compile => "frida-compile");
passthrough!(/// `frida-pm` — agent package manager (Frida.PackageManager).
    pm => "frida-pm");
passthrough!(/// `frida-join` — connect a controller session to a frida-portal.
    join => "frida-join");
passthrough!(/// `frida-itrace` — instruction-level (Stalker) trace.
    itrace => "frida-itrace");
passthrough!(/// `frida-inject` — one-shot: load a script into a PID and exit.
    inject => "frida-inject");
passthrough!(/// `frida-portal` — multi-controller portal daemon.
    portal => "frida-portal");
passthrough!(/// `gum-graft` — statically patch an on-disk binary to load a Gum agent.
    graft => "gum-graft");
passthrough!(/// `frida-strace` — cross-platform syscall tracer (17.7+).
    strace => "frida-strace");
