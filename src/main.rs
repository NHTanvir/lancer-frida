//! lancer-frida — dynamic instrumentation toolkit (Lancer ecosystem).
//!
//! Thin entry point. All logic lives in the `lancer_frida` library so the
//! companion binaries in `src/bin/` and the GUI can share it.

fn main() {
    if let Err(e) = lancer_frida::run() {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
