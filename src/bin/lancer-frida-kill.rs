//! Companion binary: `lancer-frida-kill` == `lancer-frida kill`.
//! Named with the `lancer-` prefix so it never shadows the upstream tool on PATH.

fn main() {
    if let Err(e) = lancer_frida::run_subcommand("kill") {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
