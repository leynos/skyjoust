//! Procedural macros for test fixtures that suppress lints triggered by macro
//! expansion.
//!
//! This crate is test-only tooling. It carries no runtime behaviour and is not
//! published. See [the developer's guide](../../../docs/developers-guide.md)
//! §7.3 for the policy this crate implements, and
//! [ADR 006](../../../docs/adr/006-test-macro-crate-for-fixture-expansion-lints.md)
//! for the decision record.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Item, parse_macro_input};

/// Allows the `unused_braces` lint for fixture functions.
///
/// `rstest`'s `#[fixture]` attribute re-wraps the annotated function body in a
/// further block. When the body is a single expression, that expansion trips
/// `unused_braces`, which `-D warnings` promotes to an error. Splitting the
/// body across several lines silences the lint, but `.rustfmt.toml` sets
/// `fn_single_line = true`, so `cargo fmt` collapses the repair straight back
/// into the failing form: `make check-fmt` and `make lint` then demand
/// mutually exclusive spellings of the same fixture.
///
/// This attribute breaks that deadlock. Apply it directly above `#[fixture]`,
/// leaving the fixture in its natural single-expression form.
///
/// # Examples
///
/// ```
/// use rstest::fixture;
/// use skyjoust_test_macros::allow_fixture_expansion_lints;
///
/// #[allow_fixture_expansion_lints]
/// #[fixture]
/// fn seed() -> u32 { 7 }
/// ```
///
/// # Why `allow` rather than `expect`
///
/// [The developer's guide](../../../docs/developers-guide.md) §7.3 requires
/// `#[expect(...)]` over `#[allow(...)]` at handwritten sites, so that a
/// suppression which stops applying surfaces as a warning instead of rotting.
/// That reasoning does not carry here. This attribute is applied to fixtures
/// whose bodies may or may not be single expressions, so an
/// `#[expect(unused_braces)]` would go unfulfilled — and therefore warn — on
/// every multi-statement fixture. `#[allow]` is the correct tool for a
/// suppression whose applicability varies with the annotated item.
///
/// The emitted `#[allow]` would itself trip `clippy::allow_attributes`, so the
/// expansion pairs it with a `cfg_attr(clippy, expect(...))`. The `cfg_attr`
/// guard matters: `clippy::allow_attributes` fires only under Clippy, so an
/// unguarded `#[expect]` would go unfulfilled under a plain `rustc` build.
#[proc_macro_attribute]
pub fn allow_fixture_expansion_lints(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let parsed_item = parse_macro_input!(item as Item);

    quote! {
        #[allow(
            unused_braces,
            reason = "fixture macro expansion triggers unused-braces on expression bodies"
        )]
        #[cfg_attr(
            clippy,
            expect(
                clippy::allow_attributes,
                reason = "needed to allow unused_braces for fixture macro expansion"
            )
        )]
        #parsed_item
    }
    .into()
}
