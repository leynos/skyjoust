# 006: Suppress macro-expansion lints through a test-macro crate

Status: Accepted

Date: 2026-08-15

Accepted: 2026-08-15

## Context

The workspace runs every gate with warnings denied, and `.rustfmt.toml` sets
`fn_single_line = true`. Those two settings collide with `rstest`'s `#[fixture]`
attribute.

`#[fixture]` re-wraps the annotated function body in a further block. When the
body is a single expression, the expansion trips `unused_braces`:

```plaintext
error: unnecessary braces around block return value
  --> tests/headless_scenario.rs:13:26
   = note: `-D unused-braces` implied by `-D warnings`
```

Splitting the body across several lines silences the lint. But `cargo fmt` then
collapses it straight back, because the function fits on one line. `make lint`
and `make check-fmt` therefore demand mutually exclusive spellings of the same
fixture, and neither is reachable from the other. An author who alternates
between the two forms exhausts the usual retry budget on a problem that has no
in-source resolution.

This is a known `rstest` issue rather than a defect in this repository. It will
recur at every fixture the project writes, and Skyjoust is about to acquire
many: roadmap phase 0.5 introduces behaviour-driven tests whose scenario state
is supplied by fixtures.

[The developer's guide](../developers-guide.md) §7.3 already governs lint
suppression, and it says to use `#[expect(...)]` and never `#[allow(...)]`. It
does not contemplate a lint raised by macro expansion rather than by
handwritten code.

## Decision

Add `crates/skyjoust_test_macros`, a test-only procedural-macro crate, and put
the suppression in an attribute macro:

```rust
#[allow_fixture_expansion_lints]
#[fixture]
fn seed() -> u32 { 7 }
```

This mirrors the approach already adopted elsewhere in the estate, in
`leynos/weaver`'s `weaver-test-macros`. The emitted attributes are kept
token-identical with that crate so the two do not drift.

The macro emits `#[allow(unused_braces, reason = "...")]` together with a
`#[cfg_attr(clippy, expect(clippy::allow_attributes, reason = "..."))]`.

Two details carry the reasoning:

- **`allow`, not `expect`.** The guide's preference for `expect` rests on a
  stale suppression surfacing as a warning. That argument does not transfer.
  The attribute is applied to fixtures whose bodies may or may not be single
  expressions, so an `#[expect(unused_braces)]` would go unfulfilled — and so
  warn — on every multi-statement fixture. `#[allow]` is the correct tool for a
  suppression whose applicability varies with the annotated item.
- **The `cfg_attr` guard.** The emitted `#[allow]` would itself trip
  `clippy::allow_attributes`, which the workspace denies. That lint fires only
  under Clippy, so an unguarded `#[expect]` would go unfulfilled under a plain
  `rustc` build. Gating it on `cfg(clippy)` satisfies both.

The crate is `publish = false` and carries no runtime behaviour. It is a
development dependency of the crates whose tests need it, never a normal one.

The workspace therefore grows to three members. This does not reopen
[ADR 002](002-crate-layout-and-public-api.md)'s deferral of runtime crate
splits: that decision governs *runtime* functionality, which must stay as
modules inside the runtime crate. `skyjoust-test-macros` is tooling, in the
same category as the exception ADR 002 already grants the validator crate. A
procedural macro cannot be a module of a normal crate in any case — the
compiler requires `proc-macro = true` on its own compilation unit — so no
in-crate alternative exists.

### Options considered

| Option                                            | Outcome                                                                                                  |
| ------------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| Reformulate each fixture body                     | Rejected: no formulation satisfies `make lint` and `make check-fmt` at once.                             |
| Hand-write the `allow` pair at every fixture      | Rejected: repeats a five-line attribute stack per fixture and invites drift in the reason strings.       |
| Relax `fn_single_line` or `unused_braces`         | Rejected: weakens the estate baseline repository-wide to work around one macro's expansion.              |
| Attribute macro in a test-only crate (**chosen**) | One annotation per fixture, one place to change, and consistent with `weaver-test-macros` in the estate. |

*Table 1: Options considered for resolving the fixture expansion lint.*

## Consequences

Fixtures stay in their natural single-expression form, and both formatting and
lint gates pass without either being weakened.

[The developer's guide](../developers-guide.md) §7.3 gains an explicit carve-out
so the `#[allow]` inside this crate does not read as a violation of the rule
directly above it. The rule itself is unchanged for handwritten sites.

Crates needing the attribute take a path development dependency on
`skyjoust-test-macros`. Because the dependency is dev-only and the crate is
unpublished, it does not enter any shipped artefact.

Adding suppression attributes here is deliberately narrow. A new attribute
belongs in this crate only when the lint is raised by macro expansion rather
than by handwritten code, and when no reformulation of the source satisfies
every gate at once. Everything else takes `#[expect(...)]` at the site, per
§7.3.

The crate directory is `crates/skyjoust_test_macros`, matching the underscore
convention of `crates/skyjoust_stateright_validator`. Crates destined for
extraction to their own repositories instead take their published, hyphenated
name as the directory name.
