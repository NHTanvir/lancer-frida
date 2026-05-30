//! Companion binary: `lancer-frida-rm` == `lancer-frida rm`.
//! Named with the `lancer-` prefix so it never shadows the upstream tool on PATH.

fn main() {
    if let Err(e) = lancer_frida::run_subcommand("rm") {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
