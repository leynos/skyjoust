//! Unit tests for the Bevy attribute policy.
//!
//! The shared conformance helper pins the emit, render, and "rstest is first"
//! invariants that every first-party attribute policy must satisfy.

use rstest_bdd_harness::policy_conformance::assert_attribute_policy_conformance;

use super::BevyAttributePolicy;

#[test]
fn policy_emits_only_rstest() {
    assert_attribute_policy_conformance::<BevyAttributePolicy>(&["#[rstest::rstest]"]);
}
