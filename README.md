# lancer-frida

Rust CLI wrapping the Frida dynamic instrumentation engine. Part of the Lancer ecosystem — a complete 2026 AI-era bug bounty toolkit.

## Quick start

```
setup.bat          # install Rust + LLVM (Windows, one-time)
build.bat          # build → build\lancer-frida.exe
```

## Commands

```
lancer-frida <COMMAND>

PROCESS CONTROL
  devices                                  List available devices
  ps       [-U | -R HOST]                  List running processes
  kill     [-U | -R HOST] <target>         Kill process by name or PID
  attach   [-U | -R HOST] <target> [-l FILE | -e CODE]
                                           Attach and inject script
  spawn    [-U | -R HOST] <program> [args] [-l FILE | -e CODE]
                                           Spawn and inject before entry point
  repl     [-U | -R HOST] <target>         Interactive JS REPL (like frida CLI)

INSTRUMENTATION
  trace    [-U | -R HOST] <target> -i PATTERN [-i ...]
                                           Trace function calls (frida-trace style)
  rpc list [-U | -R HOST] <target> -l FILE
                                           List rpc.exports in a script
  rpc call [-U | -R HOST] <target> [-l FILE] <function> [args...]
                                           Call rpc.exports function

ANALYSIS
  classes  [-U | -R HOST] <target> [-f FILTER] [--platform android|ios]
                                           List loaded Java/ObjC classes
  memory modules [-U|-R HOST] <target>     List loaded modules
  memory read    [-U|-R HOST] <target> <addr> [bytes]
                                           Hex-dump process memory
  memory scan    [-U|-R HOST] <target> <pattern>
                                           Scan memory for byte pattern
  discover [-U|-R HOST] <target> [-f FILTER] [--ai] [--limit N]
                                           Security-relevant API surface discovery

SCRIPT LIBRARY
  scripts list                             List bundled scripts
  scripts dump <name>                      Print bundled script source
  scripts run [-U|-R HOST] <target> <name> Run a bundled script

AI
  generate <goal> [-o FILE]               Generate Frida script with Claude
  explain  [FILE] [-e ERROR]              AI script diagnosis and fix
```

## Bundled Scripts

| Name               | Description                                         |
|--------------------|-----------------------------------------------------|
| `ssl-bypass`       | Bypass SSL/TLS certificate pinning (Android & iOS)  |
| `root-bypass`      | Bypass root/jailbreak detection                     |
| `biometric-bypass` | Bypass biometric authentication                     |
| `http-trace`       | Trace HTTP/HTTPS traffic                            |
| `anti-detect`      | Hide Frida from anti-debug/anti-Frida detection     |

## Examples

```sh
# List processes on USB device
lancer-frida ps -U

# Attach to app and dump modules
lancer-frida attach -U com.example.app

# Inject inline script
lancer-frida attach -U com.example.app -e "Java.perform(function(){ console.log('hello'); })"

# Bypass SSL pinning
lancer-frida scripts run -U com.example.app ssl-bypass

# Trace all SSL functions
lancer-frida trace -U com.example.app -i '*ssl*' -i '*SSL*'

# Interactive REPL
lancer-frida repl -U com.example.app

# Enumerate classes containing "Auth"
lancer-frida classes -U com.example.app -f Auth

# Discover security-relevant exports, AI-ranked
lancer-frida discover -U com.example.app --ai

# AI script generation
export ANTHROPIC_API_KEY=sk-ant-...
lancer-frida generate "hook okhttp3 and log every request URL" -o hook.js

# Fix a broken script
lancer-frida explain hook.js -e "TypeError: Cannot read property 'implementation' of undefined"
```

## Build

### Windows
```
setup.bat    # installs Rust + LLVM (once)
build.bat    # → build\lancer-frida.exe
```

For a packaged dist:
```
build-windows.bat   # → dist\lancer-frida-windows-x86_64\bin\lancer-frida.exe
```

### Linux / macOS
```bash
./build-linux.sh    # → build/lancer-frida
```

## Requirements
- [Rust / Cargo](https://rustup.rs)
- [LLVM](https://releases.llvm.org/) — required by `bindgen` for `frida-sys`  
  Windows: `winget install LLVM.LLVM` (or run `setup.bat`)
- (Windows) Visual C++ Build Tools 2019/2022
- `ANTHROPIC_API_KEY` env var for `generate` and `explain` commands
