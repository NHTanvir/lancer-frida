//! Companion binary: `lancer-frida-apk` == `lancer-frida apk`.
//! Named with the `lancer-` prefix so it never shadows the upstream tool on PATH.

fn main() {
    if let Err(e) = lancer_frida::run_subcommand("apk") {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
