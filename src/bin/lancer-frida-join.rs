//! Companion binary: `lancer-frida-join` == `lancer-frida join`.
//! Named with the `lancer-` prefix so it never shadows the upstream tool on PATH.

fn main() {
    if let Err(e) = lancer_frida::run_subcommand("join") {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
