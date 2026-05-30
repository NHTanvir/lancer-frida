// Reveal obfuscated/decrypted strings at runtime.
// Logs every String built from a byte[] (common output of string-decrypt
// routines) and the results of Base64 decoding, with a short backtrace so you
// can locate the decryptor.
Java.perform(function () {
    function trace(tag, value) {
        console.log("[deobfuscate] " + tag + ": " + value);
    }

    try {
        var StringCls = Java.use("java.lang.String");
        // new String(byte[]) — frequently the tail of a decrypt routine.
        StringCls.$init.overload("[B").implementation = function (bytes) {
            var result = this.$init(bytes);
            try {
                var s = StringCls.$new(bytes);
                if (s.length() > 0 && s.length() < 512) {
                    trace("new String(byte[])", s);
                }
            } catch (e) {}
            return result;
        };
        StringCls.$init.overload("[B", "java.lang.String").implementation = function (bytes, charset) {
            var result = this.$init(bytes, charset);
            try {
                trace("new String(byte[],cs)", StringCls.$new(bytes, charset));
            } catch (e) {}
            return result;
        };
    } catch (e) {}

    try {
        var Base64 = Java.use("android.util.Base64");
        Base64.decode.overload("[B", "int").implementation = function (input, flags) {
            var out = this.decode(input, flags);
            try {
                trace("Base64.decode", Java.use("java.lang.String").$new(out));
            } catch (e) {}
            return out;
        };
    } catch (e) {}

    console.log("[deobfuscate] string/Base64 hooks installed");
});
