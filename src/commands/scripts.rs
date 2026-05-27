use crate::scripts;
use crate::DeviceArgs;

pub fn list() {
    println!("{:<20} {}", "NAME", "DESCRIPTION");
    println!("{}", "-".repeat(60));
    for s in scripts::SCRIPTS {
        println!("{:<20} {}", s.name, s.description);
    }
}

pub fn dump(name: &str) -> anyhow::Result<()> {
    match scripts::find(name) {
        Some(s) => {
            println!("{}", s.source);
            Ok(())
        }
        None => anyhow::bail!("unknown script '{}' — run `scripts list` to see available scripts", name),
    }
}

pub fn run(device: &DeviceArgs, target: &str, name: &str) -> anyhow::Result<()> {
    let script = scripts::find(name)
        .ok_or_else(|| anyhow::anyhow!("unknown script '{}' — run `scripts list` to see available scripts", name))?;
    super::attach::run(device, target, None, Some(script.source))
}
