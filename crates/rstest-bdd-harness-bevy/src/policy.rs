//! Attribute policy for Bevy-backed scenarios.
//!
//! A Bevy scenario owns and advances its own [`bevy::app::App`], so the
//! generated test needs no framework-specific runner attribute. Emitting only
//! `#[rstest::rstest]` keeps the generated test free of attributes that would
//! otherwise start Bevy's own schedule runner.

use rstest_bdd_harness::{AttributePolicy, TestAttribute};

/// Attribute policy that emits only `#[rstest::rstest]`.
///
/// The policy is unit-like because the attribute list is a compile-time
/// constant; applying it costs nothing and carries no per-scenario state.
///
/// # Examples
///
/// ```
/// use rstest_bdd_harness::{AttributePolicy, TestAttribute};
/// use rstest_bdd_harness_bevy::BevyAttributePolicy;
///
/// let attributes = BevyAttributePolicy::test_attributes();
/// assert_eq!(
///     attributes,
///     &[TestAttribute::new("rstest::rstest")],
///     "the Bevy policy emits exactly the plain rstest attribute",
/// );
/// ```
pub struct BevyAttributePolicy;

/// The attribute sequence applied to every generated Bevy scenario test.
const BEVY_TEST_ATTRIBUTES: [TestAttribute; 1] = [TestAttribute::new("rstest::rstest")];

impl AttributePolicy for BevyAttributePolicy {
    fn test_attributes() -> &'static [TestAttribute] { &BEVY_TEST_ATTRIBUTES }
}

#[cfg(test)]
#[path = "policy_tests.rs"]
mod tests;
