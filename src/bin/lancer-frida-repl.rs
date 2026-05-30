//! Companion binary: `lancer-frida-repl` == `lancer-frida repl`.
//! Named with the `lancer-` prefix so it never shadows the upstream tool on PATH.

fn main() {
    if let Err(e) = lancer_frida::run_subcommand("repl") {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
