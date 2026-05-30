//! Companion binary: `lancer-frida-pull` == `lancer-frida pull`.
//! Named with the `lancer-` prefix so it never shadows the upstream tool on PATH.

fn main() {
    if let Err(e) = lancer_frida::run_subcommand("pull") {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
