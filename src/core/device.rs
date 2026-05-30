//! Typed Device / DeviceManager API (delegated to `frida-ls-devices` / `frida-ps`).

use crate::exec;

/// How a device is reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceKind {
    Local,
    Usb,
    Remote,
    Other,
}

impl DeviceKind {
    fn parse(s: &str) -> DeviceKind {
        match s.trim().to_lowercase().as_str() {
            "local" => DeviceKind::Local,
            "usb" => DeviceKind::Usb,
            "remote" => DeviceKind::Remote,
            _ => DeviceKind::Other,
        }
    }
}

/// A device reachable by the Frida engine.
#[derive(Debug, Clone)]
pub struct Device {
    pub id: String,
    pub kind: DeviceKind,
    pub name: String,
}

/// A process running on a device.
#[derive(Debug, Clone)]
pub struct Process {
    pub pid: u32,
    pub name: String,
}

/// Enumerates devices and processes via the upstream tools.
pub struct DeviceManager;

impl DeviceManager {
    /// Equivalent to `frida-ls-devices`. Parses the columnar output.
    pub fn enumerate_devices() -> anyhow::Result<Vec<Device>> {
        let out = exec::capture("frida-ls-devices", std::iter::empty::<&str>())?;
        Ok(parse_devices(&out))
    }
}

impl Device {
    /// Selector flags to target this device with a `frida-*` tool (`-D <id>`).
    pub fn selector(&self) -> Vec<String> {
        vec!["-D".to_string(), self.id.clone()]
    }

    /// Equivalent to `frida-ps -D <id>` (optionally `-a` for applications).
    pub fn enumerate_processes(&self, applications: bool) -> anyhow::Result<Vec<Process>> {
        let mut args = self.selector();
        if applications {
            args.push("-a".to_string());
        }
        let out = exec::capture("frida-ps", args)?;
        Ok(parse_processes(&out))
    }
}

/// Parse `frida-ls-devices` output: `Id  Type  Name` columns, with a separator
/// row of dashes. Heuristic whitespace splitting (≥2 spaces between columns).
fn parse_devices(out: &str) -> Vec<Device> {
    let mut devices = Vec::new();
    for line in out.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            continue;
        }
        let lower = trimmed.trim_start().to_lowercase();
        if lower.starts_with("id") || trimmed.trim_start().starts_with('-') {
            continue; // header / separator
        }
        // Split on runs of 2+ whitespace to keep multi-word names intact.
        let cols: Vec<&str> = split_columns(trimmed);
        match cols.as_slice() {
            [id, kind, name, ..] => devices.push(Device {
                id: id.to_string(),
                kind: DeviceKind::parse(kind),
                name: name.to_string(),
            }),
            [id, kind] => devices.push(Device {
                id: id.to_string(),
                kind: DeviceKind::parse(kind),
                name: String::new(),
            }),
            _ => {}
        }
    }
    devices
}

/// Parse `frida-ps` output: `PID  Name`, with header + separator rows.
fn parse_processes(out: &str) -> Vec<Process> {
    let mut procs = Vec::new();
    for line in out.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let lower = trimmed.to_lowercase();
        if lower.starts_with("pid") || trimmed.starts_with('-') {
            continue;
        }
        let cols = split_columns(trimmed);
        if let [pid, rest @ ..] = cols.as_slice() {
            if let Ok(pid) = pid.parse::<u32>() {
                procs.push(Process {
                    pid,
                    name: rest.join(" "),
                });
            }
        }
    }
    procs
}

/// Split a line on runs of two-or-more whitespace characters, preserving
/// single spaces inside a column (e.g. "Android Emulator 5554").
fn split_columns(line: &str) -> Vec<&str> {
    let mut cols = Vec::new();
    let bytes = line.as_bytes();
    let mut start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b' ' || bytes[i] == b'\t' {
            let ws_start = i;
            while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
                i += 1;
            }
            if i - ws_start >= 2 {
                if ws_start > start {
                    cols.push(line[start..ws_start].trim());
                }
                start = i;
            }
        } else {
            i += 1;
        }
    }
    if start < line.len() {
        let tail = line[start..].trim();
        if !tail.is_empty() {
            cols.push(tail);
        }
    }
    cols
}
