//! Companion binary: `lancer-frida-ls` == `lancer-frida ls`.
//! Named with the `lancer-` prefix so it never shadows the upstream tool on PATH.

fn main() {
    if let Err(e) = lancer_frida::run_subcommand("ls") {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
