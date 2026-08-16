# `skyjoust-test-macros`

Test-only procedural macros for Skyjoust. This crate carries no runtime
behaviour, is never published, and exists solely to keep test code compiling
under the estate lint baseline.

## Why it exists

The workspace denies warnings, and `.rustfmt.toml` sets `fn_single_line = true`.
`rstest`'s `#[fixture]` attribute re-wraps the annotated function body in a
further block, so a single-expression fixture trips `unused_braces`. Splitting
the body over several lines silences the lint, but `cargo fmt` then collapses
it straight back. `make check-fmt` and `make lint` end up demanding mutually
exclusive spellings of the same fixture.

`allow_fixture_expansion_lints` breaks the deadlock, so fixtures stay in their
natural form.

## Usage

Add the crate as a development dependency:

```toml
[dev-dependencies]
skyjoust-test-macros = { path = "../skyjoust_test_macros" }
```

Apply the attribute directly above `#[fixture]`:

```rust
use rstest::fixture;
use skyjoust_test_macros::allow_fixture_expansion_lints;

#[allow_fixture_expansion_lints]
#[fixture]
fn seed() -> u32 { 7 }
```

`allow_fixture_expansion_lints` accepts only `rstest` fixture functions
annotated with `#[fixture]`. Unsupported input is rejected at compile time,
rather than being passed through as though it were a fixture.

## Files

| Path                                            | Responsibility                                       |
| ----------------------------------------------- | ---------------------------------------------------- |
| `src/lib.rs`                                    | The `allow_fixture_expansion_lints` attribute macro. |
| `tests/fixture_expansion_lints.rs`              | Compile-level proof that the suppression works.      |
| `tests/fixture_expansion_lints_ui.rs`           | Trybuild harness for the macro's input contract.     |
| `tests/ui/fixture_expansion_lints/{pass,fail}/` | UI pass and compile-fail cases.                      |

*Table 1: Source layout for `skyjoust-test-macros`.*

## How it is tested

`tests/fixture_expansion_lints.rs` sets `#![deny(unused_braces)]` at the crate
level, so the file compiles only while the attribute is doing its job. Removing
the attribute from the single-expression fixture makes the test target fail to
build — that failure is the assertion. The file also covers a multi-statement
fixture, which is the case that rules out `#[expect]` in the expansion.

The UI harness covers a supported `#[fixture]` function as a compile-pass case
and unsupported input as compile-fail cases. These cases keep the macro's
compile-time input contract explicit.

Run it with:

```sh
cargo --config tools/dev-fast/config.toml test -p skyjoust-test-macros
```

## Extending

Add further suppression attributes here only when a lint is raised by *macro
expansion* rather than by handwritten code, and when no reformulation of the
source satisfies every gate at once. Handwritten sites take `#[expect(...)]`
instead; see [the developer's guide](../../docs/developers-guide.md) §7.3.
Record the reasoning in an Architecture Decision Record, as
[ADR 006](../../docs/adr/006-test-macro-crate-for-fixture-expansion-lints.md)
does for this crate.
