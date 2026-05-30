//! Companion binary: `lancer-frida-strace` == `lancer-frida strace`.
//! Named with the `lancer-` prefix so it never shadows the upstream tool on PATH.

fn main() {
    if let Err(e) = lancer_frida::run_subcommand("strace") {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
