//! Tests for the phase-tagged panic message format.
//!
//! ADR 010 makes the message a data contract: the payload mapping covers three
//! classes, and one snapshot per phase pins the field order, quoting, and
//! separators so a dropped, reordered, or requoted field fails the suite.

use std::any::Any;

use rstest::rstest;
use rstest_bdd_harness::ScenarioMetadata;

use super::{Phase, payload_text, scenario_panic_message};

/// Fixed metadata with non-default values so a formatter that drops or swaps a
/// field changes a snapshot.
fn metadata() -> ScenarioMetadata {
    ScenarioMetadata::new(
        "tests/features/demo.feature",
        "Demo scenario",
        12,
        Vec::new(),
    )
}

fn payload() -> Box<dyn Any + Send> { Box::new(String::from("boom")) }

#[rstest]
#[case::static_str(Box::new("step boom"), "step boom")]
#[case::owned_string(Box::new(String::from("step boom")), "step boom")]
#[case::opaque(Box::new(7_u32), "Box<dyn Any>")]
fn payload_text_covers_each_payload_class(
    #[case] payload: Box<dyn Any + Send>,
    #[case] expected: &str,
) {
    assert_eq!(
        payload_text(payload.as_ref()),
        expected,
        "each payload class must render as its contract text",
    );
}

#[test]
fn configure_panic_message_is_pinned() {
    let payload = payload();
    insta::assert_snapshot!(scenario_panic_message(
        Phase::Configure,
        payload.as_ref(),
        "test::Harness",
        &metadata(),
    ));
}

#[test]
fn steps_panic_message_is_pinned() {
    let payload = payload();
    insta::assert_snapshot!(scenario_panic_message(
        Phase::Steps,
        payload.as_ref(),
        "test::Harness",
        &metadata(),
    ));
}

#[test]
fn cleanup_panic_message_is_pinned() {
    let payload = payload();
    insta::assert_snapshot!(scenario_panic_message(
        Phase::Cleanup,
        payload.as_ref(),
        "test::Harness",
        &metadata(),
    ));
}
