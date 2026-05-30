//! Companion binary: `lancer-frida-graft` == `lancer-frida graft`.
//! Named with the `lancer-` prefix so it never shadows the upstream tool on PATH.

fn main() {
    if let Err(e) = lancer_frida::run_subcommand("graft") {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
