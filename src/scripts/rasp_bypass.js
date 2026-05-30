// Neutralize common mobile RASP / in-app-protection runtime checks
// (emulator, debugger, hooking-framework, integrity). Best-effort across the
// most common vendor patterns; silently skips classes that aren't present.
Java.perform(function () {
    function neuter(className, method, returnValue) {
        try {
            var cls = Java.use(className);
            cls[method].overloads.forEach(function (ov) {
                ov.implementation = function () {
                    console.log("[rasp] neutralized " + className + "." + method);
                    return returnValue;
                };
            });
        } catch (e) {}
    }

    // Debugger detection.
    neuter("android.os.Debug", "isDebuggerConnected", false);

    // Emulator / build fingerprint checks frequently route through these.
    try {
        var Build = Java.use("android.os.Build");
        Build.FINGERPRINT.value = "google/walleye/walleye:realdevice";
        Build.MODEL.value = "Pixel";
        Build.MANUFACTURER.value = "Google";
    } catch (e) {}

    // Native ptrace anti-debug.
    try {
        var ptrace = Module.findExportByName(null, "ptrace");
        if (ptrace) {
            Interceptor.replace(ptrace, new NativeCallback(function () {
                return 0;
            }, "long", ["int", "int", "pointer", "pointer"]));
            console.log("[rasp] ptrace neutralized");
        }
    } catch (e) {}

    // Generic "isRooted / isHooked / isTampered" style booleans across vendors.
    ["isRooted", "isHooked", "isTampered", "isDebuggable", "detectFrida", "checkIntegrity"].forEach(function (m) {
        Java.enumerateLoadedClasses({
            onMatch: function (name) {
                if (/security|protect|rasp|guard|integrity|root/i.test(name)) {
                    neuter(name.replace(/\//g, "."), m, false);
                }
            },
            onComplete: function () {},
        });
    });

    console.log("[rasp] bypass hooks installed");
});
