//! Companion binary: `lancer-frida-cli` — the full command-line surface,
//! never opening the GUI (plan ## GUI: the `--cli` companion).

fn main() {
    if let Err(e) = lancer_frida::run_cli() {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
