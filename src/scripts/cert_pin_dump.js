// Dump configured certificate-pinning pins (OkHttp3 CertificatePinner).
// Hooks the constructor and check() to reveal which hosts/pins an app enforces.
Java.perform(function () {
    try {
        var CertificatePinner = Java.use("okhttp3.CertificatePinner");

        // Log pins configured at construction time.
        CertificatePinner.$init.overload("java.util.Set", "okhttp3.internal.tls.CertificateChainCleaner").implementation = function (pins, cleaner) {
            console.log("[cert-pin-dump] CertificatePinner constructed with pin set:");
            try {
                var it = pins.iterator();
                while (it.hasNext()) {
                    console.log("    " + it.next().toString());
                }
            } catch (e) {}
            return this.$init(pins, cleaner);
        };

        // Log each enforced host + the expected pins as checks happen.
        CertificatePinner.check.overload("java.lang.String", "java.util.List").implementation = function (hostname, peerCertificates) {
            console.log("[cert-pin-dump] check() host=" + hostname);
            try {
                var pins = this.findMatchingPins
                    ? this.findMatchingPins(hostname)
                    : null;
                if (pins) {
                    var it = pins.iterator();
                    while (it.hasNext()) {
                        console.log("    pin: " + it.next().toString());
                    }
                }
            } catch (e) {}
            return this.check(hostname, peerCertificates);
        };

        console.log("[cert-pin-dump] hooks installed");
    } catch (e) {
        console.log("[cert-pin-dump] OkHttp not present: " + e.message);
    }
});
