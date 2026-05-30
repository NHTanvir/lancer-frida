//! Upstream parity tests (plan ### Verification).
//!
//! These verify that every upstream Frida tool the plan mandates is wired as a
//! `lancer-frida` subcommand, and that the parity matrix in `matrix.json` stays
//! in sync with the CLI. They run with `cargo test` and need neither a device
//! nor the upstream engine — they introspect the clap command tree only.

use clap::CommandFactory;
use lancer_frida::Cli;
use std::collections::HashSet;

/// Every subcommand the plan's CLI catalog + 17.x research notes require.
const EXPECTED_SUBCOMMANDS: &[&str] = &[
    // Core controller
    "devices", "ps", "kill", "attach", "spawn", "trace", "repl", "scripts",
    "classes", "memory", "discover", "rpc",
    // AI augmentation
    "generate", "explain", "summarize", "synth", "fuzz", "library", "evade", "art-update",
    // Delegated upstream tools
    "ls-devices", "pull", "push", "rm", "ls", "mkdir", "rmdir", "mv", "apk",
    "create", "compile", "pm", "join", "itrace", "inject", "portal", "graft", "strace",
    // Modes
    "self-test", "gui",
];

fn subcommand_names() -> HashSet<String> {
    Cli::command()
        .get_subcommands()
        .map(|c| c.get_name().to_string())
        .collect()
}

#[test]
fn every_expected_subcommand_is_wired() {
    let present = subcommand_names();
    let missing: Vec<&str> = EXPECTED_SUBCOMMANDS
        .iter()
        .copied()
        .filter(|s| !present.contains(*s))
        .collect();
    assert!(missing.is_empty(), "subcommands missing from the CLI: {missing:?}");
}

#[test]
fn parity_matrix_tools_are_all_wired() {
    let matrix: serde_json::Value =
        serde_json::from_str(include_str!("matrix.json")).expect("matrix.json parses");
    let present = subcommand_names();

    for tool in matrix["tools"].as_array().expect("tools is an array") {
        let upstream = tool["upstream"].as_str().unwrap_or("?");
        let sub = tool["subcommand"].as_str().expect("subcommand present");
        assert!(
            present.contains(sub),
            "upstream tool `{upstream}` is not wired as subcommand `{sub}`"
        );
    }
}

#[test]
fn cli_definition_is_internally_valid() {
    // Catches clap configuration errors (duplicate args, bad groups, etc.).
    Cli::command().debug_assert();
}
