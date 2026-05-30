// Capture protobuf/gRPC payloads at the (un)marshal boundary, before
// encryption and after decryption.
Java.perform(function () {
    // Generic protobuf-lite parse path.
    try {
        var GeneratedLite = Java.use("com.google.protobuf.GeneratedMessageLite");
        GeneratedLite.toString.implementation = function () {
            var s = this.toString();
            if (s && s.length < 4096) {
                console.log("[protobuf] " + this.getClass().getName() + " =>\n" + s);
            }
            return s;
        };
        console.log("[protobuf] GeneratedMessageLite.toString hooked");
    } catch (e) {}

    // gRPC: log method names as calls start.
    try {
        var ClientCalls = Java.use("io.grpc.stub.ClientCalls");
        ClientCalls.blockingUnaryCall.overload(
            "io.grpc.Channel", "io.grpc.MethodDescriptor", "io.grpc.CallOptions", "java.lang.Object"
        ).implementation = function (channel, method, opts, req) {
            try {
                console.log("[protobuf] gRPC call " + method.getFullMethodName() + " req=" + req);
            } catch (e) {}
            return this.blockingUnaryCall(channel, method, opts, req);
        };
        console.log("[protobuf] gRPC ClientCalls hooked");
    } catch (e) {}

    console.log("[protobuf] capture hooks installed");
});
