// SSL/TLS certificate pinning bypass
// Hooks common Android (OkHttp, TrustManager) and iOS (SecTrust) pinning APIs
Java.perform(function () {
    // OkHttp3 CertificatePinner
    try {
        var CertificatePinner = Java.use("okhttp3.CertificatePinner");
        CertificatePinner.check.overload("java.lang.String", "java.util.List").implementation = function (hostname) {
            console.log("[ssl-bypass] OkHttp3 check() bypassed for " + hostname);
        };
        CertificatePinner.check.overload("java.lang.String", "java.security.cert.Certificate").implementation = function (hostname) {
            console.log("[ssl-bypass] OkHttp3 check(cert) bypassed for " + hostname);
        };
    } catch (e) {}

    // TrustManager (accept all)
    try {
        var TrustManagerImpl = Java.use("com.android.org.conscrypt.TrustManagerImpl");
        TrustManagerImpl.verifyChain.implementation = function (untrustedChain, trustAnchorChain, host, clientAuth, ocspData, tlsSctData) {
            console.log("[ssl-bypass] TrustManagerImpl.verifyChain bypassed for " + host);
            return untrustedChain;
        };
    } catch (e) {}

    // SSLContext factory
    try {
        var SSLContext = Java.use("javax.net.ssl.SSLContext");
        var TrustAll = Java.registerClass({
            name: "lancer.TrustAll",
            implements: [Java.use("javax.net.ssl.X509TrustManager")],
            methods: {
                checkClientTrusted: function () {},
                checkServerTrusted: function () {},
                getAcceptedIssuers: function () { return []; },
            },
        });
        SSLContext.init.implementation = function (km, tm, sr) {
            console.log("[ssl-bypass] SSLContext.init — injecting trust-all TrustManager");
            this.init(km, [TrustAll.$new()], sr);
        };
    } catch (e) {}

    // WebViewClient (Android)
    try {
        var WebViewClient = Java.use("android.webkit.WebViewClient");
        WebViewClient.onReceivedSslError.implementation = function (view, handler, error) {
            console.log("[ssl-bypass] WebViewClient.onReceivedSslError — proceeding");
            handler.proceed();
        };
    } catch (e) {}

    console.log("[ssl-bypass] Android SSL bypass hooks installed");
});

// iOS SecTrustEvaluate hook
if (ObjC.available) {
    try {
        var SecTrustEvaluate = Module.findExportByName("Security", "SecTrustEvaluate");
        if (SecTrustEvaluate) {
            Interceptor.replace(SecTrustEvaluate, new NativeCallback(function (trust, result) {
                console.log("[ssl-bypass] SecTrustEvaluate bypassed");
                result.writeU32(1); // kSecTrustResultProceed
                return 0; // errSecSuccess
            }, "int", ["pointer", "pointer"]));
        }
        console.log("[ssl-bypass] iOS SecTrustEvaluate hook installed");
    } catch (e) {
        console.log("[ssl-bypass] iOS hook skipped: " + e.message);
    }
}
