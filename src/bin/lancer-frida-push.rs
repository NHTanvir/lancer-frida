//! Companion binary: `lancer-frida-push` == `lancer-frida push`.
//! Named with the `lancer-` prefix so it never shadows the upstream tool on PATH.

fn main() {
    if let Err(e) = lancer_frida::run_subcommand("push") {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
