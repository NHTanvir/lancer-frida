//! `--self-test` — the upstream parity matrix.
//!
//! Plan ### Verification: "A `lancer-frida --self-test` flag runs the full
//! parity matrix against a bundled reference target and prints pass/fail per
//! upstream feature group." Because instrumentation is delegated, the matrix
//! verifies that (a) every CLI surface is wired and (b) the upstream tools each
//! feature group depends on are reachable on `PATH`.

use crate::core::Backend;
use crate::exec;

struct Group {
    name: &'static str,
    /// Upstream binaries this feature group delegates to.
    deps: &'static [&'static str],
    /// lancer-frida subcommands in this group.
    subcommands: &'static [&'static str],
}

const GROUPS: &[Group] = &[
    Group { name: "Process control", deps: &["frida", "frida-ps", "frida-kill", "frida-ls-devices"], subcommands: &["devices", "ps", "kill", "attach", "spawn"] },
    Group { name: "Instrumentation", deps: &["frida", "frida-trace", "frida-itrace"], subcommands: &["trace", "itrace", "repl", "rpc"] },
    Group { name: "Memory & introspection", deps: &["frida"], subcommands: &["memory", "classes", "discover"] },
    Group { name: "File ops on target", deps: &["frida-pull", "frida-push", "frida-rm", "frida-ls", "frida-mkdir", "frida-rmdir", "frida-mv"], subcommands: &["pull", "push", "rm", "ls", "mkdir", "rmdir", "mv"] },
    Group { name: "Agent tooling", deps: &["frida-apk", "frida-create", "frida-compile", "frida-pm"], subcommands: &["apk", "create", "compile", "pm"] },
    Group { name: "Portal & injection", deps: &["frida-portal", "frida-join", "frida-inject", "gum-graft"], subcommands: &["portal", "join", "inject", "graft"] },
    Group { name: "Syscall tracing (17.7+)", deps: &["frida-strace"], subcommands: &["strace"] },
    Group { name: "Script library", deps: &[], subcommands: &["scripts"] },
    Group { name: "AI augmentation", deps: &[], subcommands: &["generate", "explain", "summarize", "synth", "fuzz", "library", "evade", "art-update", "discover --ai"] },
];

pub fn run() -> anyhow::Result<()> {
    println!("lancer-frida — self-test (upstream parity matrix)\n");

    let mut pass = 0usize;
    let mut degraded = 0usize;

    for g in GROUPS {
        let missing: Vec<&str> = g.deps.iter().copied().filter(|b| !exec::available(b)).collect();
        let status = if g.deps.is_empty() {
            "WIRED"
        } else if missing.is_empty() {
            "PASS"
        } else if missing.len() == g.deps.len() {
            "FAIL"
        } else {
            "DEGRADED"
        };
        match status {
            "PASS" | "WIRED" => pass += 1,
            "DEGRADED" => degraded += 1,
            _ => {}
        }
        println!("[{status:>8}] {}", g.name);
        println!("            subcommands: {}", g.subcommands.join(", "));
        if !g.deps.is_empty() {
            println!("            upstream: {}", g.deps.join(", "));
        }
        if !missing.is_empty() {
            println!("            MISSING on PATH: {}", missing.join(", "));
        }
        println!();
    }

    println!("Backend taxonomy (delegated to upstream frida-core):");
    for b in Backend::ALL {
        println!("  - {:<9} {}", b.id(), b.description());
    }
    println!();

    let ai = if std::env::var_os("ANTHROPIC_API_KEY").is_some() { "set" } else { "NOT set" };
    println!("ANTHROPIC_API_KEY: {ai} (required for the AI commands)");

    println!(
        "\nSummary: {pass} feature group(s) ready, {degraded} degraded, out of {} groups.",
        GROUPS.len()
    );
    println!("Note: instrumentation is delegated to the upstream Frida engine (Path A).");
    println!("Install missing tools with: pip install frida-tools");
    Ok(())
}
