//! Companion binary: `lancer-frida-pm` == `lancer-frida pm`.
//! Named with the `lancer-` prefix so it never shadows the upstream tool on PATH.

fn main() {
    if let Err(e) = lancer_frida::run_subcommand("pm") {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
