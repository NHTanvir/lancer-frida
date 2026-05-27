pub struct BundledScript {
    pub name: &'static str,
    pub description: &'static str,
    pub source: &'static str,
}

pub const SCRIPTS: &[BundledScript] = &[
    BundledScript {
        name: "ssl-bypass",
        description: "Bypass SSL/TLS certificate pinning (Android & iOS)",
        source: include_str!("ssl_bypass.js"),
    },
    BundledScript {
        name: "root-bypass",
        description: "Bypass root/jailbreak detection",
        source: include_str!("root_bypass.js"),
    },
    BundledScript {
        name: "biometric-bypass",
        description: "Bypass biometric authentication checks",
        source: include_str!("biometric_bypass.js"),
    },
    BundledScript {
        name: "http-trace",
        description: "Trace HTTP/HTTPS traffic via OkHttp and NSURLSession hooks",
        source: include_str!("http_trace.js"),
    },
    BundledScript {
        name: "anti-detect",
        description: "Hide Frida from anti-debug / anti-Frida detection (maps, strstr, ptrace)",
        source: include_str!("anti_detection.js"),
    },
];

pub fn find(name: &str) -> Option<&'static BundledScript> {
    SCRIPTS.iter().find(|s| s.name == name)
}
