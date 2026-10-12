//! Crate-local procedural macros used only by this harness crate's tests.
//!
//! Keeping this helper beside `rstest-bdd-harness-bevy` lets the whole
//! directory move during extraction without relying on another Skyjoust
//! workspace package.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, ItemFn, parse_macro_input};

/// Suppresses the lint emitted when `rstest` wraps a single-expression fixture.
///
/// Apply this directly above `#[fixture]` so the allowance is attached to the
/// expanded fixture instead of handwritten test logic.
///
/// # Examples
///
/// ```
/// use rstest::fixture;
/// use rstest_bdd_harness_bevy_test_macros::allow_fixture_expansion_lints;
///
/// #[allow_fixture_expansion_lints]
/// #[fixture]
/// fn seed() -> u32 { 7 }
/// ```
#[proc_macro_attribute]
pub fn allow_fixture_expansion_lints(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let fixture = parse_macro_input!(item as ItemFn);

    if !has_fixture_attribute(&fixture.attrs) {
        return syn::Error::new_spanned(
            &fixture.sig.ident,
            "expected an rstest #[fixture] function",
        )
        .to_compile_error()
        .into();
    }

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
        #fixture
    }
    .into()
}

fn has_fixture_attribute(attributes: &[Attribute]) -> bool {
    attributes
        .iter()
        .any(|attribute| attribute.path().is_ident("fixture"))
}
