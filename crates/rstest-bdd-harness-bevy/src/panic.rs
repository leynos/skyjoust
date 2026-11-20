//! Phase-tagged panic messages for Bevy scenario failures.
//!
//! ADR 010 fixes the failure contract: every scenario panic is re-raised
//! through `std::panic::panic_any` as a `String` naming the lifecycle phase,
//! the harness type, and the scenario metadata, so the panic hook prints it
//! and `#[should_panic(expected = ...)]` can match it. This module owns the
//! pure part of that contract — the payload-to-text mapping and the message
//! format — while [`crate::harness`] owns when each phase runs.

use std::{any::Any, fmt};

use rstest_bdd_harness::ScenarioMetadata;

/// The lifecycle phase whose code panicked.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Phase {
    Configure,
    Steps,
    Cleanup,
}

impl fmt::Display for Phase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Configure => "configure",
            Self::Steps => "steps",
            Self::Cleanup => "cleanup",
        })
    }
}

/// Renders a panic payload as the text the scenario message carries.
///
/// `&'static str` and `String` payloads keep their text. Every other payload
/// is reported as `Box<dyn Any>`: an arbitrary payload's `Debug` output is
/// neither stable nor guaranteed panic-free, and ADR 010 fixes this
/// placeholder as the contract.
pub(crate) fn payload_text(payload: &(dyn Any + Send)) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| {
            payload
                .downcast_ref::<&'static str>()
                .map(|text| (*text).to_owned())
        })
        .unwrap_or_else(|| "Box<dyn Any>".to_owned())
}

/// Builds the phase-tagged message ADR 010 fixes for a scenario panic.
///
/// The harness type is informative rather than a stable name: `type_name`
/// output may change between compiler versions, and consumers match on the
/// phase, feature path, line, and quoted scenario name instead.
pub(crate) fn scenario_panic_message(
    phase: Phase,
    payload: &(dyn Any + Send),
    harness_type: &str,
    metadata: &ScenarioMetadata,
) -> String {
    format!(
        "rstest-bdd-harness-bevy scenario panicked: phase={phase}, harness={harness_type}, \
         feature={feature_path}:{scenario_line}, scenario={scenario_name:?}: {text}",
        feature_path = metadata.feature_path(),
        scenario_line = metadata.scenario_line(),
        scenario_name = metadata.scenario_name(),
        text = payload_text(payload),
    )
}

#[cfg(test)]
#[path = "panic_tests.rs"]
mod tests;
