//! GUI-agnostic core library: typed host-side API over the Frida engine.
//!
//! The plan calls for a `core` library that both the CLI and the GUI sit on top
//! of, exposing typed Device / DeviceManager / Session / Script APIs and the
//! frida-core backend taxonomy. Because `lancer-frida` follows Path A
//! (delegating to the real Frida engine), these types are typed wrappers that
//! drive the upstream `frida-*` tools and parse their output.

pub mod backend;
pub mod device;
pub mod session;

pub use backend::Backend;
pub use device::{Device, DeviceKind, DeviceManager, Process};
pub use session::Session;

use std::path::PathBuf;

/// Per-OS config directory for lancer-frida. Callers create it on demand.
///
/// * Windows: `%APPDATA%\lancer-frida`
/// * Linux:   `$XDG_CONFIG_HOME/lancer-frida` or `~/.config/lancer-frida`
/// * macOS:   `~/Library/Application Support/lancer-frida`
pub fn config_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join("lancer-frida"))
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME").map(|h| {
            PathBuf::from(h)
                .join("Library")
                .join("Application Support")
                .join("lancer-frida")
        })
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        if let Some(x) = std::env::var_os("XDG_CONFIG_HOME") {
            Some(PathBuf::from(x).join("lancer-frida"))
        } else {
            std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config").join("lancer-frida"))
        }
    }
}

/// Path to the per-OS config file (`lancer-frida-config.json`).
pub fn config_file() -> Option<PathBuf> {
    config_dir().map(|d| d.join("lancer-frida-config.json"))
}
