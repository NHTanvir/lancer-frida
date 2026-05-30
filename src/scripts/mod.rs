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
    BundledScript {
        name: "cert-pin-dump",
        description: "Dump configured OkHttp CertificatePinner pins per host",
        source: include_str!("cert_pin_dump.js"),
    },
    BundledScript {
        name: "deobfuscate",
        description: "Reveal obfuscated/decrypted strings (String(byte[]), Base64)",
        source: include_str!("deobfuscate.js"),
    },
    BundledScript {
        name: "auth-headers",
        description: "Log every HTTP auth/custom header (OkHttp + HttpURLConnection)",
        source: include_str!("auth_headers.js"),
    },
    BundledScript {
        name: "protobuf-capture",
        description: "Capture protobuf/gRPC payloads at the (un)marshal boundary",
        source: include_str!("protobuf_capture.js"),
    },
    BundledScript {
        name: "rasp-bypass",
        description: "Neutralize common RASP/in-app-protection runtime checks",
        source: include_str!("rasp_bypass.js"),
    },
];

pub fn find(name: &str) -> Option<&'static BundledScript> {
    SCRIPTS.iter().find(|s| s.name == name)
}
