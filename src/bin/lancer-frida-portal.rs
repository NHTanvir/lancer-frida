//! Companion binary: `lancer-frida-portal` == `lancer-frida portal`.
//! Named with the `lancer-` prefix so it never shadows the upstream tool on PATH.

fn main() {
    if let Err(e) = lancer_frida::run_subcommand("portal") {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
