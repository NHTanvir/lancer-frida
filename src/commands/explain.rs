use anyhow::Context;
use std::io::{self, Read};

use super::generate::call_claude_with_system;
use serde_json::json;

const SYSTEM_PROMPT: &str = r#"You are an expert Frida dynamic instrumentation engineer and debugger.
The user will provide a Frida JavaScript script and optionally an error message or crash report.
Diagnose the problem and provide:
1. Root cause explanation (1-3 sentences)
2. Fixed script (complete, ready to use)
3. What you changed and why (brief bullet list)
No markdown fences around the fixed script."#;

pub fn run(
    script_file: Option<&std::path::Path>,
    error: Option<&str>,
) -> anyhow::Result<()> {
    let source = match script_file {
        Some(path) => std::fs::read_to_string(path)
            .with_context(|| format!("reading script file {}", path.display()))?,
        None => {
            eprintln!("[explain] No script file given — reading from stdin...");
            let mut s = String::new();
            io::stdin().read_to_string(&mut s)?;
            s
        }
    };

    if source.trim().is_empty() {
        anyhow::bail!("script is empty");
    }

    let prompt = if let Some(err) = error {
        format!("Script:\n{}\n\nError/crash:\n{}", source, err)
    } else {
        format!(
            "Script:\n{}\n\nNo specific error provided — review the script for correctness, \
common pitfalls, and potential issues.",
            source
        )
    };

    eprintln!("[explain] Asking Claude to diagnose the script...");

    let system = json!([{
        "type": "text",
        "text": SYSTEM_PROMPT,
        "cache_control": { "type": "ephemeral" }
    }]);

    let response = call_claude_with_system(&prompt, Some(system))?;

    if response.trim().is_empty() {
        anyhow::bail!("Claude returned no response");
    }

    println!("{}", response);
    Ok(())
}
