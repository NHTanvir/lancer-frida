// HTTP/HTTPS traffic tracer — OkHttp3, HttpURLConnection, NSURLSession
Java.perform(function () {
    // OkHttp3 RealCall.execute
    try {
        var OkHttpClient = Java.use("okhttp3.OkHttpClient");
        var Request = Java.use("okhttp3.Request");
        var RealCall = Java.use("okhttp3.internal.connection.RealCall");
        RealCall.execute.implementation = function () {
            var req = this.request();
            var url = req.url().toString();
            var method = req.method();
            console.log("[http-trace] -> " + method + " " + url);
            var resp = this.execute();
            if (resp) {
                console.log("[http-trace] <- " + resp.code() + " " + url);
            }
            return resp;
        };
    } catch (e) {}

    // OkHttp3 RealCall.enqueue (async)
    try {
        var RealCall2 = Java.use("okhttp3.internal.connection.RealCall");
        RealCall2.enqueue.implementation = function (cb) {
            var req = this.request();
            var url = req.url().toString();
            var method = req.method();
            console.log("[http-trace] -> async " + method + " " + url);
            return this.enqueue(cb);
        };
    } catch (e) {}

    // HttpURLConnection
    try {
        var HttpURLConnection = Java.use("java.net.HttpURLConnection");
        HttpURLConnection.getResponseCode.implementation = function () {
            var url = this.getURL().toString();
            var code = this.getResponseCode();
            console.log("[http-trace] <- " + code + " " + url);
            return code;
        };
    } catch (e) {}

    console.log("[http-trace] Android HTTP hooks installed");
});

// iOS NSURLSession
if (ObjC.available) {
    try {
        var NSURLSession = ObjC.classes.NSURLSession;
        if (NSURLSession) {
            var dataTask = NSURLSession["- dataTaskWithRequest:completionHandler:"];
            Interceptor.attach(dataTask.implementation, {
                onEnter: function (args) {
                    var req = ObjC.Object(args[2]);
                    var url = req.URL().absoluteString().toString();
                    var method = req.HTTPMethod().toString();
                    console.log("[http-trace] -> " + method + " " + url);
                },
            });
        }
        console.log("[http-trace] iOS NSURLSession hooks installed");
    } catch (e) {
        console.log("[http-trace] iOS hook skipped: " + e.message);
    }
}
