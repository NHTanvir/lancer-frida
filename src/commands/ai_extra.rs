//! Additional AI-augmentation commands beyond `generate`/`explain`/`discover`.
//!
//! Covers the remaining plan "AI augmentation ideas":
//!   - `summarize` — trace-to-summary (idea #6)
//!   - `synth`     — symbolic bypass synthesizer (idea #2)
//!   - `fuzz`      — auto-fuzz seed from observed inputs (idea #9)
//!   - `library`   — hook-script library with semantic search (idea #8)
//!
//! All route through the shared Claude client in `commands::generate`.

use super::generate::call_claude_with_system;
use anyhow::Context;
use serde_json::json;
use std::io::{self, Read, Write};
use std::path::Path;

fn read_input(file: Option<&Path>, what: &str) -> anyhow::Result<String> {
    match file {
        Some(path) => std::fs::read_to_string(path)
            .with_context(|| format!("reading {} {}", what, path.display())),
        None => {
            eprintln!("[{what}] reading from stdin...");
            let mut s = String::new();
            io::stdin().read_to_string(&mut s)?;
            Ok(s)
        }
    }
}

fn system(text: &str) -> serde_json::Value {
    json!([{ "type": "text", "text": text, "cache_control": { "type": "ephemeral" } }])
}

const SUMMARIZE_SYSTEM: &str = r#"You are a reverse-engineering analyst. The user pastes a Frida/Stalker
trace or instrumentation log. Produce a concise plain-English summary: what the traced code does,
notable side effects, interesting arguments/return values, and anything security-relevant
(crypto, auth, network, secrets). Use short bullet points. No code unless quoting a key value."#;

pub fn summarize(input: Option<&Path>) -> anyhow::Result<()> {
    let trace = read_input(input, "summarize")?;
    if trace.trim().is_empty() {
        anyhow::bail!("nothing to summarize");
    }
    eprintln!("[summarize] asking Claude to summarize {} bytes of trace...", trace.len());
    let resp = call_claude_with_system(&trace, Some(system(SUMMARIZE_SYSTEM)))?;
    println!("{resp}");
    Ok(())
}

const SYNTH_SYSTEM: &str = r#"You are an expert Frida engineer specializing in defeating mobile app
defenses (SSL pinning, root/jailbreak detection, anti-debug, RASP, integrity checks). The user
describes a specific defense. Output ONLY a complete, ready-to-run Frida JavaScript bypass script
for Frida 17.x — no prose, no markdown fences. Use Java.perform() for Android paths and
ObjC.available guards for iOS. Prefix log lines with '[synth]'."#;

pub fn synth(description: &str, output: Option<&Path>) -> anyhow::Result<()> {
    eprintln!("[synth] synthesizing a bypass for: {description}");
    let script = call_claude_with_system(description, Some(system(SYNTH_SYSTEM)))?;
    if script.trim().is_empty() {
        anyhow::bail!("Claude returned no script");
    }
    match output {
        Some(path) => {
            std::fs::write(path, &script).context("writing script")?;
            eprintln!("[synth] written to {}", path.display());
        }
        None => {
            io::stdout().write_all(script.as_bytes())?;
            println!();
        }
    }
    Ok(())
}

const FUZZ_SYSTEM: &str = r#"You are a fuzzing seed generator. Given a set of legitimate, observed
inputs (one per line), produce malformed/edge-case mutations likely to trigger parser or
validation bugs: boundary values, oversized fields, encoding tricks, injection metacharacters,
type confusion, truncation. Output one mutated seed per line, nothing else."#;

pub fn fuzz(input: Option<&Path>, count: usize) -> anyhow::Result<()> {
    let inputs = read_input(input, "fuzz")?;
    if inputs.trim().is_empty() {
        anyhow::bail!("no observed inputs provided");
    }
    eprintln!("[fuzz] requesting {count} mutated seeds...");
    let prompt = format!("Generate {count} mutated fuzz seeds from these observed inputs:\n{inputs}");
    let resp = call_claude_with_system(&prompt, Some(system(FUZZ_SYSTEM)))?;
    println!("{resp}");
    Ok(())
}

const EVADE_SYSTEM: &str = r#"You are an anti-detection specialist. The user's target app has detected Frida
or instrumentation. Given their description of HOW it detected them (or a log), suggest the next
layer of evasion as a concrete, ready-to-run Frida script plus a one-line rationale per hook.
Cover: /proc/self/maps scanning, named-pipe/port checks, ptrace, thread-name scans, strstr("frida"),
and code-integrity hashing. Output the script first (no fences), then rationale bullets."#;

/// Anti-detection patch suggester (plan AI idea #7).
pub fn evade(symptom: &str, output: Option<&Path>) -> anyhow::Result<()> {
    eprintln!("[evade] suggesting evasion for: {symptom}");
    let resp = call_claude_with_system(symptom, Some(system(EVADE_SYSTEM)))?;
    if resp.trim().is_empty() {
        anyhow::bail!("Claude returned nothing");
    }
    match output {
        Some(path) => {
            std::fs::write(path, &resp).context("writing script")?;
            eprintln!("[evade] written to {}", path.display());
        }
        None => println!("{resp}"),
    }
    Ok(())
}

const ARTUPDATE_SYSTEM: &str = r#"You are an ART-internals expert. Given a Frida Java-hooking script that
fails on a newer Android version, and the Android version(s) involved, explain what changed in ART
(ArtMethod layout, JIT flags, invocation paths, hidden-API enforcement) and produce an updated script
that works on the new version. Be specific about the AOSP change. Output the analysis, then the
updated script (no fences)."#;

/// Per-version ART hook updater (plan AI idea #5).
pub fn artupdate(android_version: &str, input: Option<&Path>) -> anyhow::Result<()> {
    let script = read_input(input, "artupdate")?;
    if script.trim().is_empty() {
        anyhow::bail!("provide the failing Java-hook script (file or stdin)");
    }
    eprintln!("[artupdate] adapting hook for Android {android_version}...");
    let prompt = format!("Target Android version: {android_version}\nFailing script:\n{script}");
    let resp = call_claude_with_system(&prompt, Some(system(ARTUPDATE_SYSTEM)))?;
    println!("{resp}");
    Ok(())
}

const LIBRARY_SYSTEM: &str = r#"You are a Frida script librarian. Given a catalog of available scripts
(name + description) and a user's natural-language need, pick the best matching script(s) and explain
in one sentence why. If none fit well, say so and suggest using `lancer-frida generate` or `synth`.
Be concise."#;

pub fn library(query: &str) -> anyhow::Result<()> {
    // Build a catalog from the bundled scripts plus any user scripts in the
    // per-OS config dir (~/.config/lancer-frida/scripts or %APPDATA%).
    let mut catalog = String::from("Bundled scripts:\n");
    for s in crate::scripts::SCRIPTS {
        catalog.push_str(&format!("  {} — {}\n", s.name, s.description));
    }
    if let Some(dir) = crate::core::config_dir() {
        let scripts_dir = dir.join("scripts");
        if let Ok(entries) = std::fs::read_dir(&scripts_dir) {
            catalog.push_str("Local scripts:\n");
            for e in entries.flatten() {
                if let Some(name) = e.file_name().to_str() {
                    catalog.push_str(&format!("  {name}\n"));
                }
            }
        }
    }

    eprintln!("[library] searching the script library for: {query}");
    let prompt = format!("{catalog}\nUser need: {query}");
    let resp = call_claude_with_system(&prompt, Some(system(LIBRARY_SYSTEM)))?;
    println!("{resp}");
    Ok(())
}
