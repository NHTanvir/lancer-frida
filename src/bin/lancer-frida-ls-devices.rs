//! Companion binary: `lancer-frida-ls-devices` == `lancer-frida ls-devices`.
//! Named with the `lancer-` prefix so it never shadows the upstream tool on PATH.

fn main() {
    if let Err(e) = lancer_frida::run_subcommand("ls-devices") {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
