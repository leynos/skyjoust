# Skyjoust developer's guide

This guide is for maintainers changing the Skyjoust runtime contract,
Stateright validator, trace tools, and accompanying specifications.

## 1. Normative references

The validator must stay synchronized with these source documents:

- [Product requirements](skyjoust-product-requirements.md)
- [Technical design](skyjoust-technical-design.md)
- [State graph specification](skyjoust-state-graphs.yaml)
- [Validator contract](../crates/skyjoust_stateright_validator/spec/validator_contract.yaml)
- [Architecture decision records](adr/)

## 2. Runtime crate and module boundary

[ADR 002](adr/002-crate-layout-and-public-api.md) is the source of truth for
the workspace shape. The accepted decision is one runtime crate with strict
internal modules, beside the separate `skyjoust_stateright_validator` crate.
Maintainers should add new runtime functionality as a module inside the runtime
crate, not as a new crate.

The runtime modules and their responsibilities follow the technical design's
runtime ownership table: `game_app`, `core`, `sim`, `terrain`, `stategraphs`,
`render`, `audio`, `ui`, and `assets`. The dependency direction is one-way:

```plaintext
game_app -> subsystem modules -> core
```

Lower-level modules must not call higher-level orchestration or adapters.
Domain code must not depend on adapters, renderer code, audio backends, process
setup, or window lifecycle glue.

A module may be promoted to its own crate only when at least one of these
conditions holds:

- an API is reused across the runtime, developer tooling, or the validator
  crate;
- a boundary is stable enough to test and release independently of the
  runtime crate.

Record any such extraction in a follow-up ADR before changing `Cargo.toml`
workspace members.

## 3. Validator module structure

The `skyjoust-stateright-validator` crate keeps domain logic in small modules:

| File                    | Responsibility                                                           |
| ----------------------- | ------------------------------------------------------------------------ |
| `actions.rs`            | Domain action and small enum definitions used by traces and transitions. |
| `action_generation.rs`  | Legal action enumeration for each state during model exploration.        |
| `ceremonies.rs`         | Tournament, duel, wedding, banquet, and consequence transitions.         |
| `ledgers.rs`            | Score and reward ledger state.                                           |
| `model.rs`              | Core bounded model configuration.                                        |
| `properties.rs`         | `always` invariants and `sometimes` reachability checks.                 |
| `scoring.rs`            | Score atoms, morale changes, winner selection, and reward tallying.      |
| `serde_impls/`          | Serialization adapters for domain types, one module per adapter group.   |
| `state.rs`              | Core state snapshot, state enums, and guard helpers.                     |
| `stateright_adapter.rs` | Stateright `Model` implementation for the core model.                    |
| `trace.rs`              | Concrete JSON trace replay and validation output types.                  |
| `transitions.rs`        | Top-level transition dispatcher and gameplay/reward transitions.         |

The binary `src/bin/validate_trace.rs` is process glue. The Explorer example in
`examples/serve_explorer.rs` is diagnostic glue.

### 3.1. `serde_impls/action_names.rs` boundary

`serde_impls/action_names.rs` owns the canonical JSON name tables and lookups
for unit `SkyAction` variants: `UNIT_ACTION_NAMES`, `TAGGED_ACTION_NAMES`,
`unit_action_name`, and `unit_action_from_name`. `unit_action_name` returns
`Option<&'static str>`, `None` for a tagged variant, so the one call site in
`serde_impls/actions.rs` can turn that into a `serde::ser::Error` rather than
panicking; `unit_action_from_name` returns `Option<SkyAction>`, `None` for an
unrecognized name. These exist to back the `SkyAction` serde adapter in
`serde_impls/actions.rs`, the module's only permitted consumer — its
`pub(super)` visibility enforces that at compile time. Domain modules and
external callers must not depend on this private serialization detail; go
through `Serialize`/`Deserialize` for `SkyAction` instead.

Composition rule: when a `SkyAction` variant changes, update the
serializer/deserializer dispatch in `serde_impls/actions.rs` and the name
tables or lookup functions in `action_names.rs` together — whether that is a
name-table entry for a unit variant or a `serialize_tagged`/
`serialize_team_action` call for a payload-carrying one. Reuse policy: this
module is scoped to `SkyAction` serde name conversion only; give another wire
format or domain type its own adapter-owned mapping rather than extending this
one.

## 4. Public application programming interface

The crate root re-exports the public surface used by tests, tools, and future
runtime integration:

| Symbol                                         | Purpose                                                              |
| ---------------------------------------------- | -------------------------------------------------------------------- |
| `SkyjoustInteractionModel`                     | Configures the bounded interaction model.                            |
| `SkyState`                                     | Carries the complete model snapshot.                                 |
| `SkyAction`                                    | Represents one replayable high-level action.                         |
| `TraceValidation`                              | Reports trace replay success or failure.                             |
| `TraceFailure`                                 | Describes the first failed replay step.                              |
| `validate_trace`                               | Replays concrete actions against transition guards and invariants.   |
| State enums                                    | Expose app, match, ceremony, Warfront, objective, and reward phases. |
| `ALWAYS_PROPERTIES` and `SOMETIMES_PROPERTIES` | Expose property tables for diagnostics.                              |

Serde support is intentionally isolated in the `serde_impls` module; domain
modules do not derive serialization traits directly.

## 5. Extending the model

Model changes should be made in this order:

1. Add the `SkyAction` variant in `actions.rs`.
2. Add action generation in `action_generation.rs` so Stateright can explore
   the new transition only from legal states.
3. Add transition handling in `transitions.rs` or the appropriate feature
   module, such as `ceremonies.rs`.
4. Add or update invariants and reachability checks in `properties.rs`.
5. Update the canonical graph bundle in `docs/skyjoust-state-graphs.yaml`,
   then run `make generate-state-graphs` to regenerate
   `docs/skyjoust-state-graphs.json`.
6. Update
   `crates/skyjoust_stateright_validator/spec/validator_contract.yaml`.
7. Add a focused unit test and, when structured output, user-interface output,
   diagnostics, or JSON contracts change, an `insta` snapshot test with
   meaningful, stable assertions.
8. Add `trybuild` coverage when the change introduces compile-time behaviour,
   such as macro expansion, trait bounds, feature-gated APIs, or compile-pass
   and compile-fail contracts.

Guard helpers should stay pure and side-effect free. Transition helpers may
mutate only the cloned destination state supplied by the caller.

## 6. Local validation

Run the full Rust gates before committing code:

```sh
make check-fmt
make check-state-graphs
make lint
make typecheck
make test
cargo doc --no-deps --workspace
```

`make typecheck` runs `cargo check --workspace --all-targets --all-features`
with `RUSTFLAGS="-D warnings "`, so warnings fail the gate.

Run Markdown checks after documentation changes:

```sh
make fmt
make markdownlint
make nixie
git diff --check
```

### 6.1. Markdown lint configuration

Markdown validation uses two configuration files that must stay aligned:

- `.markdownlint-cli2.jsonc` configures `make markdownlint`, which runs
  `markdownlint-cli2` across repository Markdown.
- `.markdownlint.json` configures the `markdownlint --fix` step invoked by
  `mdformat-all` during `make fmt`.

Both files enforce the same rule choices:

- `MD004` requires dash bullets, matching the documentation style guide.
- `MD010` permits hard tabs inside code blocks while still checking prose.
- `MD013` wraps prose at 80 columns, allows code blocks up to 120 columns, and
  ignores table and heading widths.
- `MD029` requires ordered lists to use increasing numeric markers.

Keeping these files synchronized prevents `make fmt` from applying one Markdown
policy while `make markdownlint` checks another. When a Markdown lint rule
changes, update both files in the same commit and rerun `make fmt` and
`make markdownlint`.

### 6.2. Spelling policy

Run `make spelling` to enforce en-GB-oxendict spelling in tracked Markdown
prose. The generated and tracked `typos.toml` starts from the shared estate
dictionary. The shared `typos-config-builder` CLI refreshes an untracked local
cache only when the authoritative copy is newer, so a valid tracked
configuration remains usable in a network-restricted checkout.

Keep repository-specific identifiers and deliberate quotations in
`typos.local.toml`. Run `make spelling-config-write` to regenerate the tracked
configuration and `make spelling-config` to verify it. Never edit generated
entries by hand.

The Stateright Explorer can help diagnose counterexamples:

```sh
cargo run -p skyjoust-stateright-validator --example serve_explorer
```

The trace validator can replay a fixture:

```sh
cargo run -p skyjoust-stateright-validator --bin validate_trace \
  < crates/skyjoust_stateright_validator/traces/tournament_reward_commit.json
```

Set `SKYJOUST_VALIDATOR_DEBUG=1` during debug builds to print transition
attempts during depth-first search.

## 7. Lint baseline

Skyjoust follows the df12 estate's phase 2 Rust baseline for lint configuration.
`Cargo.toml` is the source of truth for the exact lint set; this section
explains where the tables live and how to work with them, not what every entry
does.

### 7.1. Table placement and inheritance

The canonical clippy, rust, and rustdoc lint tables live under
`[workspace.lints.clippy]`, `[workspace.lints.rust]`, and
`[workspace.lints.rustdoc]` in the root `Cargo.toml`. Every workspace member,
including the root `skyjoust` package, inherits them with:

```toml
[lints]
workspace = true
```

Any new crate added to the workspace must carry that stanza. A crate without it
silently opts out of the estate baseline instead of failing loudly, so review
new `Cargo.toml` files for it during code review.

### 7.2. What the tables enforce

The tables summarize the estate's phase 2 baseline; read `Cargo.toml` for the
authoritative, current list rather than relying on this summary. In brief:

- Clippy denies panic-prone operations (`unwrap_used`, `expect_used`,
  `indexing_slicing`, `unreachable`, and similar), debugging leftovers
  (`dbg_macro`, `print_stdout`, `print_stderr`), numerical foot-guns such as
  lossy casts, and direct environment access (`disallowed_methods`, see §7.4).
  `clippy::pedantic` runs at `warn`.
- The rust lint set forbids `unsafe_code` outright and denies `missing_docs`,
  so every public item needs a doc comment.
- The rustdoc set denies broken and private intra-doc links, bare URLs, and
  malformed code blocks, alongside `missing_crate_level_docs`.

### 7.3. Silencing a lint

Fix the violation. When a fix is not worthwhile — the site is deliberately
outside the mandate, or a rewrite would cost more clarity than it buys —
annotate it with:

```rust
#[expect(clippy::some_lint, reason = "why this site is a sanctioned exception")]
```

Never use `#[allow(...)]` for this. `expect` only suppresses the lint while the
violation still exists; once the site is fixed or refactored away, the
unfulfilled expectation itself becomes a warning, so a stale annotation
surfaces instead of rotting silently.

### 7.4. `clippy.toml` thresholds and the environment-access mandate

`clippy.toml` sets the code-health thresholds (cognitive complexity, argument
count, function length, nesting depth) and lists the `std::env` functions
clippy disallows: `var`, `var_os`, `vars`, `vars_os`, `set_var`, and
`remove_var`. Inject an environment reader (or, in tests, a stub environment)
instead of reading or mutating the process environment directly.
`allow-expect-in-tests` permits `.expect(...)` inside `#[test]` functions, but
not in shared, non-test helpers.

### 7.5. Toolchain

`rust-toolchain.toml` pins a dated nightly channel and requires it to supply the
`rustfmt`, `clippy`, and `rust-analyzer` components (a repository may carry
additional components beyond these three, such as an opt-in Cranelift codegen
backend for `tools/dev-fast/config.toml`). `cargo fmt` and `cargo clippy` both
run under the pinned nightly automatically; rustup resolves the toolchain from
`rust-toolchain.toml` without an explicit `+nightly-...` argument.
