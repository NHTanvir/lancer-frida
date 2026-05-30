# CLAUDE.md

Guidance for Claude Code (claude.ai/code) when working in this repository.

## What this project is

`lancer-frida` is a Rust **controller + native GUI** over the [Frida](https://frida.re)
dynamic-instrumentation engine — part of the "Lancer" bug-bounty toolkit. It is
**not** a from-scratch reimplementation of Frida. It follows "Path A": the real
Frida engine is reached by **delegating to the upstream `frida-*` command-line
tools** (installed via `pip install frida-tools`), while this crate adds a
unified CLI, a native GUI, an AI layer (Claude), a bundled script library, and
cross-Lancer-tool integration.

Engine internals (frida-gum: Stalker, relocators, the in-process JS engine, ART
hooking) are intentionally delegated, not rewritten. Do not attempt to
reimplement them — that is explicitly out of scope.

## Build & run

There is no devkit/`frida-sys` dependency — the build needs only the Rust
toolchain (and a C/C++ toolchain for transitive crates / eframe on the host OS).

```bash
# Windows
build.bat                 # -> build\lancer-frida.exe
build-windows.bat         # packaged dist -> dist\lancer-frida-windows-x86_64\

# Linux
./build-linux.sh          # -> build/lancer-frida

# macOS
./build-mac.sh            # -> build/lancer-frida (+ optional .app via cargo-bundle)

# Direct cargo
cargo build --release                      # GUI included (default)
cargo build --release --no-default-features  # CLI-only (no eframe/egui)
cargo test  --test upstream_parity         # CLI-parity tests (no device needed)
```

**Runtime dependency:** the upstream Frida tools must be on `PATH`
(`pip install frida-tools`). The `generate`/`explain`/`discover --ai`/
`summarize`/`synth`/`fuzz`/`library`/`evade`/`art-update` commands also need
`ANTHROPIC_API_KEY`.

## Running it

- Bare `lancer-frida` (no args) → launches the **native GUI**.
- `lancer-frida --cli` or any subcommand → command-line surface.
- `lancer-frida --self-test` → prints the upstream parity matrix (which feature
  groups' tools are present on `PATH`).

## Architecture

Single crate exposing a **library** (`lancer_frida`) that the main binary, the
GUI, and 24 companion binaries all share.

```
src/
  lib.rs            # clap Cli/Commands + dispatch(); run()/run_cli()/run_subcommand()
  main.rs           # thin entry point -> lancer_frida::run()
  exec.rs           # run()/capture()/available() — spawn upstream frida-* tools
  device.rs         # device_flags(DeviceArgs) -> ["-U"] / ["-H", host]
  script.rs         # load_source(file|eval) for attach/spawn/rpc
  commands/         # one module per CLI subcommand (mostly subprocess delegation)
    tools.rs        #   pass-through wrappers (pull/push/.../strace) — args forwarded verbatim
    generate.rs     #   Claude API client: call_claude / call_claude_with_system
    ai_extra.rs     #   summarize/synth/fuzz/library/evade/art-update
    discover.rs     #   enumerate exports + optional Claude ranking (--ai)
  core/             # GUI-agnostic typed host API
    device.rs       #   Device/DeviceManager/Process (parses frida-ls-devices / frida-ps)
    session.rs      #   Session::run_script (one-shot `frida -e ... --no-pause`)
    backend.rs      #   Backend enum: Droidy/Darwin/Linux/Windows/Freebsd/Qnx/Fruity/Barebone/Simmy
  gui/              # eframe/egui frontend (feature = "gui", default on)
    app.rs          #   three-pane LancerApp; background work via threads + mpsc
    server.rs       #   local HTTP integration server (see below)
  selftest.rs       # --self-test parity matrix
  scripts/          # bundled Frida JS, embedded via include_str! in mod.rs
  bin/              # 24 companion binaries: each calls run_subcommand("<sub>")
tests/upstream_parity/   # matrix.json + tests asserting every tool is wired
```

### Key patterns

- **Subprocess delegation.** Almost every command builds a `std::process::Command`
  for a `frida-*` binary (or uses `core::Session`/`exec`). Device selection flags
  come from `device::device_flags`. New tool wrappers should forward args verbatim
  (`commands/tools.rs` style) for automatic upstream-flag parity.
- **Companion binaries.** `src/bin/lancer-frida-<tool>.rs` is a one-liner calling
  `lancer_frida::run_subcommand("<sub>")`. They are named `lancer-frida-*` (not
  `frida-*`) so they never shadow the upstream tools on `PATH`. Add new ones by
  adding a file here; Cargo auto-detects them.
- **GUI integration server.** When the GUI runs it starts a localhost HTTP server
  on an auto-allocated port written to `%TEMP%/lancer-frida.port`, exposing
  `GET /v1/state`, `POST /v1/import`, `GET /v1/export?format=json|csv|sarif|markdown`
  for other Lancer tools.
- **AI commands** all route through `commands::generate::call_claude_with_system`
  (blocking `reqwest`, prompt caching). Keep that the single Claude entry point.

### Adding a CLI subcommand

1. Add a variant to `Commands` in `src/lib.rs` (clap derives kebab-case names).
2. Add a match arm in `dispatch()`.
3. Implement it in `src/commands/<name>.rs` and register the module in
   `src/commands/mod.rs`.
4. If it maps to an upstream tool, add it to `tests/upstream_parity/matrix.json`
   and the `EXPECTED_SUBCOMMANDS` list so the parity test covers it.
5. Optionally add a `src/bin/lancer-frida-<name>.rs` companion.

## Conventions

- Commit messages: conventional style (`feat:`/`fix:`/`docs:`). **Never** add
  `Co-Authored-By` or "Generated with Claude Code" trailers.
- `plan.md` is the private local blueprint — it is gitignored and must **never**
  be committed or pushed.
- The code targets `eframe = "0.29"` (egui 0.29 APIs, e.g. `ScrollArea::id_source`).
  If bumping egui, update the GUI accordingly.
