use anyhow::Context;
use reqwest::blocking::Client;
use serde_json::{json, Value};
use std::io::{self, Write};

const CLAUDE_API_URL: &str = "https://api.anthropic.com/v1/messages";
const MODEL: &str = "claude-opus-4-7";

const SYSTEM_PROMPT: &str = r#"You are an expert Frida dynamic instrumentation engineer.
When the user describes a goal, output ONLY the Frida JavaScript script that achieves it — no prose, no markdown fences, no explanation.
The script must work with Frida 17.x. Use Java.perform() for Android, ObjC.available checks for iOS.
Always add console.log() statements prefixed with '[lancer]' to report key intercept events."#;

pub fn run(goal: &str, output: Option<&std::path::Path>) -> anyhow::Result<()> {
    let system = json!([{
        "type": "text",
        "text": SYSTEM_PROMPT,
        "cache_control": { "type": "ephemeral" }
    }]);

    eprintln!("[generate] Calling Claude ({MODEL}) for: {goal}");

    let script = call_claude_with_system(goal, Some(system))?;

    if script.trim().is_empty() {
        anyhow::bail!("Claude returned no text content");
    }

    match output {
        Some(path) => {
            std::fs::write(path, &script).context("writing script to file")?;
            eprintln!("[generate] Script written to {}", path.display());
        }
        None => {
            io::stdout().write_all(script.as_bytes())?;
            println!();
        }
    }

    Ok(())
}

/// General-purpose Claude call — returns the text response.
pub fn call_claude(prompt: &str) -> anyhow::Result<String> {
    call_claude_with_system(prompt, None)
}

/// Call Claude with an optional structured system prompt (for caching).
pub fn call_claude_with_system(
    prompt: &str,
    system: Option<Value>,
) -> anyhow::Result<String> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .context("ANTHROPIC_API_KEY environment variable not set")?;

    let client = Client::new();

    let mut body = json!({
        "model": MODEL,
        "max_tokens": 8192,
        "thinking": { "type": "adaptive" },
        "messages": [{ "role": "user", "content": prompt }]
    });

    if let Some(sys) = system {
        body["system"] = sys;
    }

    let resp = client
        .post(CLAUDE_API_URL)
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .context("HTTP request to Claude API failed")?;

    let status = resp.status();
    let text = resp.text().context("reading Claude API response body")?;

    if !status.is_success() {
        anyhow::bail!("Claude API error {}: {}", status, text);
    }

    let json: Value = serde_json::from_str(&text).context("parsing Claude API JSON")?;

    let result = if let Some(blocks) = json["content"].as_array() {
        blocks
            .iter()
            .filter(|b| b["type"].as_str() == Some("text"))
            .filter_map(|b| b["text"].as_str())
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        String::new()
    };

    Ok(result)
}
