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
}

impl ErrorRecorder {
    /// Returns a snapshot of the events recorded so far.
    pub fn events(&self) -> Vec<RecordedEvent> {
        self.events
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
pub fn capture_errors<T>(run: impl FnOnce() -> T) -> (T, Vec<RecordedEvent>) {
    let recorder = ErrorRecorder::default();
    let observer = recorder.clone();
    let value = tracing::subscriber::with_default(recorder, || {
        tracing::callsite::rebuild_interest_cache();
        run()
    });
    (value, observer.events())
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
        if let Some(recorded) = visitor.into_event() {
            self.events
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(recorded);
        }
    }

    fn enter(&self, _span: &Id) {}

    fn exit(&self, _span: &Id) {}
}

/// Collects the three fields the failure contract names.
#[derive(Default)]
struct FieldVisitor {
    phase: Option<String>,
    scenario_name: Option<String>,
    message: Option<String>,
}

impl FieldVisitor {
    fn into_event(self) -> Option<RecordedEvent> {
        Some(RecordedEvent {
            phase: self.phase?,
            scenario_name: self.scenario_name?,
            message: self.message?,
        })
    }

    fn record_field(&mut self, field: &Field, value: String) {
        match field.name() {
            "phase" => self.phase = Some(value),
            "scenario_name" => self.scenario_name = Some(value),
            "error" => self.message = Some(value),
            _ => {}
        }
    }
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
