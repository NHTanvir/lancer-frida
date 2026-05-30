// Log every HTTP header added via OkHttp's Request.Builder — surfaces auth
// tokens, API keys, signatures and custom headers.
Java.perform(function () {
    try {
        var Builder = Java.use("okhttp3.Request$Builder");
        Builder.addHeader.overload("java.lang.String", "java.lang.String").implementation = function (name, value) {
            console.log("[auth-headers] addHeader  " + name + ": " + value);
            return this.addHeader(name, value);
        };
        Builder.header.overload("java.lang.String", "java.lang.String").implementation = function (name, value) {
            console.log("[auth-headers] header     " + name + ": " + value);
            return this.header(name, value);
        };
        console.log("[auth-headers] OkHttp Request.Builder hooks installed");
    } catch (e) {
        console.log("[auth-headers] OkHttp not present: " + e.message);
    }

    // Also catch java.net.HttpURLConnection.setRequestProperty.
    try {
        var Url = Java.use("java.net.HttpURLConnection");
        Url.setRequestProperty.implementation = function (key, value) {
            console.log("[auth-headers] setRequestProperty " + key + ": " + value);
            return this.setRequestProperty(key, value);
        };
    } catch (e) {}
});
