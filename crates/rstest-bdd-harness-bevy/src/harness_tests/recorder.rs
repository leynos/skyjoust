//! Handwritten `tracing` subscriber that records scenario error events.
//!
//! ADR 010's lifecycle table counts `tracing::error!` records per outcome row,
//! so the lifecycle tests observe events instead of log text. The subscriber
//! keeps only the fields the contract names — `phase`, `scenario_name`, and the
//! `error` message — and is installed for the duration of one run with
//! `tracing::subscriber::with_default`.

use std::{
    fmt,
    sync::{Arc, Mutex, PoisonError},
};

use tracing::{
    Event,
    Level,
    Metadata,
    Subscriber,
    field::{Field, Visit},
    level_filters::LevelFilter,
    span::{Attributes, Id, Record},
    subscriber::Interest,
};

/// One captured `ERROR` event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordedEvent {
    pub phase: String,
    pub scenario_name: String,
    pub message: String,
}

/// Records `ERROR` events in a shared vector that outlives the subscription.
#[derive(Clone, Default)]
pub struct ErrorRecorder {
    events: Arc<Mutex<Vec<RecordedEvent>>>,
    malformed: Arc<Mutex<Vec<String>>>,
}

impl ErrorRecorder {
    /// Returns a snapshot of the events recorded so far.
    pub fn events(&self) -> Vec<RecordedEvent> {
        self.events
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn malformed_events(&self) -> Vec<String> {
        self.malformed
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

/// Runs `run` with an [`ErrorRecorder`] installed and returns its events.
///
/// The callsite interest cache is global, and the test binary may have emitted
/// the same callsite before under the no-op default subscriber; rebuilding the
/// cache after installing the recorder makes the event reach the recorder on
/// every run rather than only the first.
///
/// # Panics
///
/// Panics when an `ERROR` event names the harness field but omits a contract
/// field: such a record would otherwise be invisible to a test that asserts
/// no records, letting a malformed diagnostic pass as a clean run.
pub fn capture_errors<T>(run: impl FnOnce() -> T) -> (T, Vec<RecordedEvent>) {
    let recorder = ErrorRecorder::default();
    let observer = recorder.clone();
    let value = tracing::subscriber::with_default(recorder, || {
        tracing::callsite::rebuild_interest_cache();
        run()
    });
    let malformed = observer.malformed_events();
    assert!(
        malformed.is_empty(),
        "a harness record is missing contract fields: {malformed:?}"
    );
    (value, observer.events())
}

/// One captured `DEBUG` start event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartRecord {
    pub harness_type: String,
    pub feature_path: String,
    pub scenario_name: String,
    pub scenario_line: String,
    pub message: String,
}

/// Records the harness's `DEBUG` start events in a shared vector.
#[derive(Clone, Default)]
pub struct StartRecorder {
    records: Arc<Mutex<Vec<StartRecord>>>,
}

/// Runs `run` with a [`StartRecorder`] installed and returns its records.
///
/// The interest-cache treatment mirrors [`capture_errors`], because the start
/// event's callsite is also shared across runs.
pub fn capture_start<T>(run: impl FnOnce() -> T) -> (T, Vec<StartRecord>) {
    let recorder = StartRecorder::default();
    let observer = recorder.clone();
    let value = tracing::subscriber::with_default(recorder, || {
        tracing::callsite::rebuild_interest_cache();
        run()
    });
    let records = observer
        .records
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone();
    (value, records)
}

impl Subscriber for StartRecorder {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool { metadata.level() == &Level::DEBUG }

    fn max_level_hint(&self) -> Option<LevelFilter> { Some(LevelFilter::DEBUG) }

    fn register_callsite(&self, _metadata: &Metadata<'_>) -> Interest { Interest::sometimes() }

    fn new_span(&self, _span: &Attributes<'_>) -> Id { Id::from_u64(1) }

    fn record(&self, _span: &Id, _values: &Record<'_>) {}

    fn record_follows_from(&self, _span: &Id, _follows: &Id) {}

    fn event(&self, event: &Event<'_>) {
        if event.metadata().level() != &Level::DEBUG {
            return;
        }
        let mut visitor = StartVisitor::default();
        event.record(&mut visitor);
        if let Some(record) = visitor.finish() {
            self.records
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(record);
        }
    }

    fn enter(&self, _span: &Id) {}

    fn exit(&self, _span: &Id) {}
}

/// Collects the fields the start event promises.
#[derive(Default)]
struct StartVisitor {
    harness_type: Option<String>,
    feature_path: Option<String>,
    scenario_name: Option<String>,
    scenario_line: Option<String>,
    message: Option<String>,
}

impl StartVisitor {
    /// Returns a record only when every promised field is present.
    fn finish(self) -> Option<StartRecord> {
        Some(StartRecord {
            harness_type: self.harness_type?,
            feature_path: self.feature_path?,
            scenario_name: self.scenario_name?,
            scenario_line: self.scenario_line?,
            message: self.message?,
        })
    }

    fn record_field(&mut self, field: &Field, value: String) {
        match field.name() {
            "harness_type" => self.harness_type = Some(value),
            "feature_path" => self.feature_path = Some(value),
            "scenario_name" => self.scenario_name = Some(value),
            "scenario_line" => self.scenario_line = Some(value),
            "message" => self.message = Some(value),
            _ => {}
        }
    }
}

impl Visit for StartVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.record_field(field, value.to_owned());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.record_field(field, value.to_string());
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.record_field(field, value.to_string());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.record_field(field, value.to_string());
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.record_field(field, format!("{value:?}"));
    }
}

impl Subscriber for ErrorRecorder {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool { metadata.level() == &Level::ERROR }

    fn max_level_hint(&self) -> Option<LevelFilter> { Some(LevelFilter::ERROR) }

    fn register_callsite(&self, _metadata: &Metadata<'_>) -> Interest {
        // The recorder is installed per run, and several runs share the global
        // callsite cache; `sometimes` makes every event re-evaluate `enabled`
        // against the subscriber active on this thread.
        Interest::sometimes()
    }

    fn new_span(&self, _span: &Attributes<'_>) -> Id { Id::from_u64(1) }

    fn record(&self, _span: &Id, _values: &Record<'_>) {}

    fn record_follows_from(&self, _span: &Id, _follows: &Id) {}

    fn event(&self, event: &Event<'_>) {
        if event.metadata().level() != &Level::ERROR {
            return;
        }
        let mut visitor = FieldVisitor::default();
        event.record(&mut visitor);
        match visitor.classify() {
            CapturedEvent::Complete(recorded) => self
                .events
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(recorded),
            CapturedEvent::Malformed(reason) => self
                .malformed
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(reason),
            CapturedEvent::Foreign => {}
        }
    }

    fn enter(&self, _span: &Id) {}

    fn exit(&self, _span: &Id) {}
}

/// How one `ERROR` event relates to the harness failure contract.
enum CapturedEvent {
    /// A harness record carrying every contract field.
    Complete(RecordedEvent),
    /// A record naming the harness field but missing a contract field.
    Malformed(String),
    /// An event from some other producer, which this recorder ignores.
    Foreign,
}

/// Collects the fields the failure contract names, plus the harness marker.
#[derive(Default)]
struct FieldVisitor {
    phase: Option<String>,
    scenario_name: Option<String>,
    error: Option<String>,
    harness_type: Option<String>,
    event_message: Option<String>,
}

impl FieldVisitor {
    /// Classifies the event so a malformed harness record cannot be dropped.
    ///
    /// The harness always emits `harness_type` beside the three contract
    /// fields, so a record with the marker but without a field is a broken
    /// harness record rather than another producer's event. Defaulting the
    /// missing field would let a malformed record pass as a clean run.
    fn classify(self) -> CapturedEvent {
        let Self {
            phase,
            scenario_name,
            error,
            harness_type,
            event_message,
        } = self;
        match (phase, scenario_name, error) {
            (Some(phase_text), Some(name_text), Some(error_text)) => {
                CapturedEvent::Complete(RecordedEvent {
                    phase: phase_text,
                    scenario_name: name_text,
                    message: error_text,
                })
            }
            (phase_slot, name_slot, error_slot) => {
                if harness_type.is_none() {
                    CapturedEvent::Foreign
                } else {
                    CapturedEvent::Malformed(missing_fields(
                        phase_slot.as_deref(),
                        name_slot.as_deref(),
                        error_slot.as_deref(),
                        event_message.as_deref(),
                    ))
                }
            }
        }
    }

    fn record_field(&mut self, field: &Field, value: String) {
        match field.name() {
            "phase" => self.phase = Some(value),
            "scenario_name" => self.scenario_name = Some(value),
            "error" => self.error = Some(value),
            "harness_type" => self.harness_type = Some(value),
            "message" => self.event_message = Some(value),
            _ => {}
        }
    }
}

/// Describes a harness record that omitted one or more contract fields.
fn missing_fields(
    phase: Option<&str>,
    scenario_name: Option<&str>,
    error: Option<&str>,
    event_message: Option<&str>,
) -> String {
    let mut missing = Vec::new();
    if phase.is_none() {
        missing.push("phase");
    }
    if scenario_name.is_none() {
        missing.push("scenario_name");
    }
    if error.is_none() {
        missing.push("error");
    }
    format!("missing required fields {missing:?} in ERROR event {event_message:?}")
}

impl Visit for FieldVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.record_field(field, value.to_owned());
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        // `%value` fields arrive through the `Display`-to-`Debug` adapter, whose
        // `Debug` output is the display text without quotes.
        self.record_field(field, format!("{value:?}"));
    }
}

#[cfg(test)]
mod tests {
    //! Tests proving a malformed harness record cannot pass as a clean run.

    use super::capture_errors;

    #[test]
    #[should_panic(expected = "missing required fields")]
    fn a_harness_record_missing_a_contract_field_is_reported() {
        capture_errors(|| {
            tracing::error!(
                harness_type = "test::Harness",
                phase = "steps",
                scenario_name = "Demo scenario",
                "a record without an error field"
            );
        });
    }

    #[test]
    fn a_foreign_error_event_is_ignored() {
        let (value, events) = capture_errors(|| {
            tracing::error!("an unrelated producer");
            7_u32
        });
        assert_eq!(value, 7, "the observed run's value is returned");
        assert_eq!(
            events.len(),
            0,
            "an event without the harness marker is not a harness record"
        );
    }
}
