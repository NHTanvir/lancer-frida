use std::process::Command;

pub fn run() -> anyhow::Result<()> {
    let status = Command::new("frida-ls-devices").status()?;
    if !status.success() {
        anyhow::bail!("frida-ls-devices exited with {}", status);
    }
    Ok(())
}
