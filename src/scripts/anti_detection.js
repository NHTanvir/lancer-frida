// Anti-Frida / anti-debug detection bypass
// Hooks /proc/self/maps reads, strstr calls, and ptrace to hide Frida's presence

// --- Native hooks ---
var fopen = Module.findExportByName(null, 'fopen');
var fgets = Module.findExportByName(null, 'fgets');
var strstr = Module.findExportByName(null, 'strstr');
var ptrace = Module.findExportByName(null, 'ptrace');

var FRIDA_STRINGS = ['frida', 'gum-js-loop', 'gmain', 'linjector', 'frida-agent'];

if (fopen) {
    Interceptor.attach(fopen, {
        onEnter: function (args) {
            this.path = args[0].readUtf8String();
        },
        onLeave: function (retval) {
            if (this.path && this.path.indexOf('/proc/self/maps') !== -1) {
                console.log('[anti-detect] fopen(/proc/self/maps) intercepted');
                retval.replace(ptr(0)); // return NULL — as if open failed
            }
        },
    });
}

if (fgets) {
    Interceptor.attach(fgets, {
        onLeave: function (retval) {
            if (!retval.isNull()) {
                try {
                    var line = retval.readUtf8String();
                    for (var i = 0; i < FRIDA_STRINGS.length; i++) {
                        if (line && line.toLowerCase().indexOf(FRIDA_STRINGS[i]) !== -1) {
                            console.log('[anti-detect] fgets() — hiding Frida line: ' + line.trim());
                            retval.writeUtf8String('\n'); // replace with blank line
                            break;
                        }
                    }
                } catch (_) {}
            }
        },
    });
}

if (strstr) {
    Interceptor.attach(strstr, {
        onEnter: function (args) {
            try {
                this.needle = args[1].readUtf8String();
            } catch (_) {
                this.needle = null;
            }
        },
        onLeave: function (retval) {
            if (this.needle) {
                var needleLow = this.needle.toLowerCase();
                for (var i = 0; i < FRIDA_STRINGS.length; i++) {
                    if (needleLow === FRIDA_STRINGS[i]) {
                        console.log('[anti-detect] strstr() searching for "' + this.needle + '" -> null');
                        retval.replace(ptr(0));
                        break;
                    }
                }
            }
        },
    });
}

if (ptrace) {
    Interceptor.attach(ptrace, {
        onLeave: function (retval) {
            // PTRACE_TRACEME (0) returns -1 if already being traced
            // Replace -1 with 0 to fool anti-debug checks
            if (retval.toInt32() === -1) {
                console.log('[anti-detect] ptrace() returned -1, spoofing to 0');
                retval.replace(ptr(0));
            }
        },
    });
}

// Android: hook /proc/<pid>/status reads looking for "TracerPid"
var open = Module.findExportByName(null, 'open');
var read = Module.findExportByName(null, 'read');

if (open && read) {
    var _tracerFds = {};
    Interceptor.attach(open, {
        onEnter: function (args) {
            try {
                this.path = args[0].readUtf8String();
            } catch (_) { this.path = ''; }
        },
        onLeave: function (retval) {
            if (this.path && this.path.indexOf('/status') !== -1) {
                _tracerFds[retval.toInt32()] = true;
            }
        },
    });

    Interceptor.attach(read, {
        onEnter: function (args) {
            this.fd = args[0].toInt32();
            this.buf = args[1];
        },
        onLeave: function (retval) {
            if (_tracerFds[this.fd] && retval.toInt32() > 0) {
                try {
                    var content = this.buf.readUtf8String(retval.toInt32());
                    if (content.indexOf('TracerPid') !== -1) {
                        var patched = content.replace(/TracerPid:\s*\d+/, 'TracerPid:\t0');
                        console.log('[anti-detect] TracerPid spoofed in /proc/*/status');
                        this.buf.writeUtf8String(patched);
                    }
                } catch (_) {}
            }
        },
    });
}

console.log('[anti-detect] Anti-Frida/anti-debug hooks installed');
