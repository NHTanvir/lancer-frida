# Upstream parity tests

These tests pin `lancer-frida`'s CLI surface to the upstream Frida tool suite
(baseline **Frida 17.9.8**) so CI fails if a tool is dropped or renamed.

- **`matrix.json`** — the source of truth: every upstream tool, the
  `lancer-frida` subcommand it maps to, whether it is delegated to the upstream
  binary, and representative flag combinations.
- **`main.rs`** — runnable tests (`cargo test --test upstream_parity`):
  - `every_expected_subcommand_is_wired` — all mandated subcommands exist.
  - `parity_matrix_tools_are_all_wired` — `matrix.json` stays in sync with the CLI.
  - `cli_definition_is_internally_valid` — clap's own `debug_assert`.

## What "parity" means here

`lancer-frida` follows the plan's **Path A**: the instrumentation engine is the
real Frida, reached by delegating to the `frida-*` binaries. For delegated tools
the wrappers forward every argument verbatim, so upstream-flag parity is
structural — there is no per-flag mirror to drift. These tests therefore verify
*reachability and wiring*; the live behaviour is the upstream engine's.

`lancer-frida --self-test` runs the runtime side of the matrix: it checks which
upstream binaries are actually present on `PATH` and prints pass/fail per
feature group.
