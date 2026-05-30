//! Companion binary: `lancer-frida-rmdir` == `lancer-frida rmdir`.
//! Named with the `lancer-` prefix so it never shadows the upstream tool on PATH.

fn main() {
    if let Err(e) = lancer_frida::run_subcommand("rmdir") {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
