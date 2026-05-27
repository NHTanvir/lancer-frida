// Root / jailbreak detection bypass
Java.perform(function () {
    // RootBeer and generic file-existence checks
    var rootFiles = [
        "/system/app/Superuser.apk",
        "/system/xbin/su",
        "/system/bin/su",
        "/sbin/su",
        "/data/local/xbin/su",
        "/data/local/bin/su",
        "/data/local/su",
        "/system/sd/xbin/su",
        "/system/bin/.ext/.su",
        "/system/usr/we-need-root/su-backup",
        "/system/xbin/mu",
    ];

    var File = Java.use("java.io.File");
    File.exists.implementation = function () {
        var name = this.getAbsolutePath();
        for (var i = 0; i < rootFiles.length; i++) {
            if (name === rootFiles[i]) {
                console.log("[root-bypass] File.exists() -> false for " + name);
                return false;
            }
        }
        return this.exists();
    };

    // Runtime.exec (su check)
    var Runtime = Java.use("java.lang.Runtime");
    Runtime.exec.overload("java.lang.String").implementation = function (cmd) {
        if (cmd.indexOf("su") !== -1) {
            console.log("[root-bypass] Runtime.exec(su) blocked");
            throw Java.use("java.io.IOException").$new("Permission denied");
        }
        return this.exec(cmd);
    };

    // RootBeer isRooted
    try {
        var RootBeer = Java.use("com.scottyab.rootbeer.RootBeer");
        RootBeer.isRooted.implementation = function () {
            console.log("[root-bypass] RootBeer.isRooted() -> false");
            return false;
        };
        RootBeer.isRootedWithoutBusyBoxCheck.implementation = function () { return false; };
    } catch (e) {}

    console.log("[root-bypass] Root detection bypass hooks installed");
});

// iOS jailbreak bypass
if (ObjC.available) {
    try {
        // Hook fileExistsAtPath
        var NSFileManager = ObjC.classes.NSFileManager;
        if (NSFileManager) {
            var orig = NSFileManager["- fileExistsAtPath:"];
            Interceptor.attach(orig.implementation, {
                onEnter: function (args) {
                    this.path = ObjC.Object(args[2]).toString();
                },
                onLeave: function (retval) {
                    var jailbreakPaths = ["/Applications/Cydia.app", "/bin/bash", "/usr/sbin/sshd", "/etc/apt"];
                    for (var i = 0; i < jailbreakPaths.length; i++) {
                        if (this.path === jailbreakPaths[i]) {
                            console.log("[root-bypass] fileExistsAtPath -> false for " + this.path);
                            retval.replace(0);
                        }
                    }
                },
            });
        }
        console.log("[root-bypass] iOS jailbreak bypass hooks installed");
    } catch (e) {
        console.log("[root-bypass] iOS hook skipped: " + e.message);
    }
}
