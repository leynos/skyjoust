# 007: Adopt the Rust build standard in `.cargo/config.toml`

Status: Accepted

Date: 2026-10-02

Accepted: 2026-10-02

## Context

The estate Rust build standard (concordat rule `rust-build-defaults`, BD-001 to
BD-006) makes the parallel `rustc` frontend, `mold` on Linux, and Cranelift
where the whole suite passes the defaults for development builds, committed to
`.cargo/config.toml` so that Cargo auto-discovers them.

Skyjoust previously kept those settings only in the explicit
`tools/dev-fast/config.toml` fragment and told contributors never to copy them
into `.cargo/config.toml`, because Cargo applies that file to every build.

## Decision

`.cargo/config.toml` carries the standard:

- every `rustflags` source names `-Zthreads=8`, and the
  `cfg(target_os = "linux")` source adds `mold`;
- the development profile selects Cranelift, and `[unstable] codegen-backend`
  enables the key.

Release and coverage are held off it deliberately. `make release` assigns
`RUSTFLAGS`, which displaces every `rustflags` source. Coverage holds the
development profile on LLVM, because `-Cinstrument-coverage` is LLVM-only.
`make lint` runs Whitaker on LLVM, because its driver builds outside the
workspace configuration. The Makefile restates the flags wherever it assigns
`RUSTFLAGS`, composing them with an inherited value. The dev-fast fragment
remains for explicit opt-in use.

## Consequences

- A bare `cargo build` gets the standard; `AGENTS.md` no longer forbids the
  settings in `.cargo/config.toml`.
- A Linux host needs `mold` installed before any build.
- `tests/build_standard_contract.rs` holds the configuration and the Makefile
  recipes to this decision, and `tests/build_standard_ci.rs` holds the CI
  install order.
- CI runs `make test` before coverage, so the whole suite is exercised under
  Cranelift; coverage itself holds the development profile on LLVM.
