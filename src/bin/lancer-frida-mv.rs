//! Companion binary: `lancer-frida-mv` == `lancer-frida mv`.
//! Named with the `lancer-` prefix so it never shadows the upstream tool on PATH.

fn main() {
    if let Err(e) = lancer_frida::run_subcommand("mv") {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
