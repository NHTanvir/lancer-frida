//! lancer-frida
//! Rust clone of frida. See plan.md (local, gitignored) for the spec.

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "lancer-frida", version, about = "Rust clone of frida")]
struct Cli {}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let _cli = Cli::parse();
    println!("lancer-frida - scaffold ready. Implement per plan.md.");
    Ok(())
}
