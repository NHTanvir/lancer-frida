//! frida-core backend taxonomy (plan ### Backend taxonomy + 17.x research notes).
//!
//! These are the platform backends the upstream frida-core organises its
//! per-platform glue into. `lancer-frida` exposes them as a typed enum so the
//! GUI and `--self-test` can present/verify the full matrix even though the
//! actual instrumentation is delegated to the upstream engine.

/// A frida-core backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// Android — ADB transport, zygote/zymbiote instrumentation, ART hooking.
    Droidy,
    /// iOS / macOS / visionOS / watchOS / tvOS — Mach, dyld, ObjC, Swift.
    Darwin,
    /// Linux — ptrace + dlopen, eBPF SyscallTracer, spawn gating.
    Linux,
    /// Windows — CreateRemoteThread, NTAPI, PE injection (incl. ARM64).
    Windows,
    /// FreeBSD — x86 (32-bit, 17.3) + amd64.
    Freebsd,
    /// QNX — newer paths via Barebone.
    Qnx,
    /// Host-side USB/USBMUX/NCM transport to Darwin devices.
    Fruity,
    /// "No OS" — bare-metal / hypervisor / QEMU / RISC-V / XNU-in-QEMU.
    Barebone,
    /// Apple simulators via CoreSimulator.framework (17.3).
    Simmy,
}

impl Backend {
    /// Every backend, in plan order.
    pub const ALL: &'static [Backend] = &[
        Backend::Droidy,
        Backend::Darwin,
        Backend::Linux,
        Backend::Windows,
        Backend::Freebsd,
        Backend::Qnx,
        Backend::Fruity,
        Backend::Barebone,
        Backend::Simmy,
    ];

    /// Lowercase identifier (matches frida-core naming).
    pub fn id(self) -> &'static str {
        match self {
            Backend::Droidy => "droidy",
            Backend::Darwin => "darwin",
            Backend::Linux => "linux",
            Backend::Windows => "windows",
            Backend::Freebsd => "freebsd",
            Backend::Qnx => "qnx",
            Backend::Fruity => "fruity",
            Backend::Barebone => "barebone",
            Backend::Simmy => "simmy",
        }
    }

    /// One-line human description.
    pub fn description(self) -> &'static str {
        match self {
            Backend::Droidy => "Android (ADB, zygote/zymbiote, ART hooking)",
            Backend::Darwin => "iOS/macOS/visionOS/watchOS/tvOS (Mach, dyld, ObjC, Swift)",
            Backend::Linux => "Linux (ptrace+dlopen, eBPF SyscallTracer, spawn gating)",
            Backend::Windows => "Windows (CreateRemoteThread, NTAPI, PE; x86_64 + ARM64)",
            Backend::Freebsd => "FreeBSD (x86 32-bit + amd64)",
            Backend::Qnx => "QNX (via Barebone code paths)",
            Backend::Fruity => "Host-side USB/USBMUX/NCM transport to Darwin devices",
            Backend::Barebone => "Bare-metal / hypervisor / QEMU / RISC-V / XNU-in-QEMU",
            Backend::Simmy => "Apple simulators via CoreSimulator.framework",
        }
    }
}
