//! Companion binary: `lancer-frida-trace` == `lancer-frida trace`.
//! Named with the `lancer-` prefix so it never shadows the upstream tool on PATH.

fn main() {
    if let Err(e) = lancer_frida::run_subcommand("trace") {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
