//! Test profiles that record lifecycle hooks and trip on unexpected calls.
//!
//! Every profile is an uninhabited unit-like enum because `BevyProfile`'s
//! hooks are associated functions: the harness never constructs a profile
//! value. Tripwire hooks use `std::panic::panic_any` rather than `panic!`
//! because Clippy's `panic_in_result_fn` forbids panic macros inside a
//! `Result`-returning hook.

use std::{fmt, panic::panic_any, rc::Rc};

use bevy::{
    app::{App, Plugin},
    ecs::{resource::Resource, system::ResMut},
    prelude::{FixedUpdate, MinimalPlugins},
    time::TimeUpdateStrategy,
};
use rstest_bdd_harness::{HarnessError, HarnessResult, ScenarioMetadata};

use crate::{BevyProfile, BevyScenario};

/// Counts cleanup hook invocations inside the shared app.
#[derive(Resource, Default)]
pub struct CleanupCalls(pub u32);

/// Counts runner closure invocations inside the shared app.
#[derive(Resource, Default)]
pub struct RunnerCalls(pub u32);

/// Counts fixed updates observed by a profile-installed system.
#[derive(Resource, Default)]
pub struct FixedTicks(pub u32);

/// Profile whose hooks succeed and whose cleanup counts its calls.
pub enum Recording {}

impl BevyProfile for Recording {
    fn configure(app: &mut App, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CleanupCalls>();
        app.init_resource::<RunnerCalls>();
        Ok(())
    }

    fn cleanup(scenario: &BevyScenario, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
        scenario.with_world(|world| {
            world.resource_mut::<CleanupCalls>().0 += 1;
        });
        Ok(())
    }
}

/// Profile whose configure hook fails before the runner can run.
pub enum ConfigureFails {}

impl BevyProfile for ConfigureFails {
    fn configure(_app: &mut App, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
        Err(HarnessError::RuntimeBuildFailed(std::io::Error::other(
            "configure sentinel",
        )))
    }

    fn cleanup(_scenario: &BevyScenario, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
        panic_any("tripwire: cleanup must not run after a configure error");
    }
}

/// Profile whose configure hook panics before the runner can run.
pub enum ConfigurePanics {}

impl BevyProfile for ConfigurePanics {
    fn configure(_app: &mut App, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
        panic_any("configure boom");
    }

    fn cleanup(_scenario: &BevyScenario, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
        panic_any("tripwire: cleanup must not run after a configure panic");
    }
}

/// Resource whose drop panics, exercising the harness's guarded drops.
#[derive(Resource)]
pub struct PanicOnDrop;

impl Drop for PanicOnDrop {
    fn drop(&mut self) { panic_any("drop boom"); }
}

/// Panic payload whose own destructor panics.
///
/// The harness must transcribe and forget a caught secondary payload rather
/// than drop it, or this destructor replaces the original phase-tagged panic.
pub struct PanicOnPayloadDrop;

impl Drop for PanicOnPayloadDrop {
    fn drop(&mut self) { panic_any(String::from("payload drop boom")); }
}

/// Resource whose drop panics with [`PanicOnPayloadDrop`].
#[derive(Resource)]
pub struct PanicOnDropWithHostilePayload;

impl Drop for PanicOnDropWithHostilePayload {
    fn drop(&mut self) { panic_any(PanicOnPayloadDrop); }
}

/// Profile whose configure hook installs [`PanicOnDrop`] and then panics.
pub enum ConfigurePanicsWithDrop {}

impl BevyProfile for ConfigurePanicsWithDrop {
    fn configure(app: &mut App, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
        app.insert_resource(PanicOnDrop);
        panic_any("configure boom");
    }
}

/// Profile whose configure hook installs [`PanicOnDrop`] and then fails.
pub enum ConfigureFailsWithDrop {}

impl BevyProfile for ConfigureFailsWithDrop {
    fn configure(app: &mut App, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
        app.insert_resource(PanicOnDrop);
        Err(HarnessError::RuntimeBuildFailed(std::io::Error::other(
            "configure sentinel",
        )))
    }
}

/// Profile whose configure hook installs [`PanicOnDrop`] and then succeeds.
///
/// The runner returns normally, so the application is released on the harness's
/// success path rather than while a panic unwinds.
pub enum SucceedsWithDrop {}

impl BevyProfile for SucceedsWithDrop {
    fn configure(app: &mut App, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
        app.insert_resource(PanicOnDrop);
        Ok(())
    }
}

/// Profile whose configure hook installs a hostile drop and then panics.
pub enum ConfigurePanicsWithHostileDrop {}

impl BevyProfile for ConfigurePanicsWithHostileDrop {
    fn configure(app: &mut App, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
        app.insert_resource(PanicOnDropWithHostilePayload);
        panic_any("configure boom");
    }
}

/// Profile whose cleanup panics with a payload that panics on drop.
pub enum CleanupPanicsWithHostilePayload {}

impl BevyProfile for CleanupPanicsWithHostilePayload {
    fn configure(app: &mut App, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CleanupCalls>();
        app.init_resource::<RunnerCalls>();
        Ok(())
    }

    fn cleanup(_scenario: &BevyScenario, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
        panic_any(PanicOnPayloadDrop);
    }
}

/// Profile whose cleanup increments the counter and then panics.
pub enum CleanupPanics {}

impl BevyProfile for CleanupPanics {
    fn configure(app: &mut App, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CleanupCalls>();
        app.init_resource::<RunnerCalls>();
        Ok(())
    }

    fn cleanup(scenario: &BevyScenario, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
        scenario.with_world(|world| {
            world.resource_mut::<CleanupCalls>().0 += 1;
        });
        panic_any("cleanup boom");
    }
}

/// Profile whose cleanup increments the counter and then fails.
pub enum CleanupFails {}

impl BevyProfile for CleanupFails {
    fn configure(app: &mut App, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CleanupCalls>();
        app.init_resource::<RunnerCalls>();
        Ok(())
    }

    fn cleanup(scenario: &BevyScenario, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
        scenario.with_world(|world| {
            world.resource_mut::<CleanupCalls>().0 += 1;
        });
        Err(HarnessError::RuntimeBuildFailed(std::io::Error::other(
            "cleanup sentinel",
        )))
    }
}

/// Error whose `Display` panics.
///
/// A profile may return any error, so the harness must treat its formatting as
/// hostile code rather than assume it is safe to render.
pub struct PanicOnDisplay;

impl fmt::Display for PanicOnDisplay {
    fn fmt(&self, _formatter: &mut fmt::Formatter<'_>) -> fmt::Result { panic_any("display boom") }
}

impl fmt::Debug for PanicOnDisplay {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PanicOnDisplay")
    }
}

impl std::error::Error for PanicOnDisplay {}

/// Error whose destructor panics.
///
/// The harness must dispose of a profile's error inside its catch boundary, or
/// the destructor replaces the panic it was trying to record.
pub struct PanicOnErrorDrop;

impl fmt::Display for PanicOnErrorDrop {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("error drop boom")
    }
}

impl fmt::Debug for PanicOnErrorDrop {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PanicOnErrorDrop")
    }
}

impl std::error::Error for PanicOnErrorDrop {}

impl Drop for PanicOnErrorDrop {
    fn drop(&mut self) { panic_any("error drop boom"); }
}

/// Profile whose cleanup fails with an error that panics when displayed.
pub enum CleanupFailsWithHostileDisplay {}

impl BevyProfile for CleanupFailsWithHostileDisplay {
    fn configure(app: &mut App, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CleanupCalls>();
        app.init_resource::<RunnerCalls>();
        Ok(())
    }

    fn cleanup(scenario: &BevyScenario, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
        scenario.with_world(|world| {
            world.resource_mut::<CleanupCalls>().0 += 1;
        });
        Err(HarnessError::RuntimeBuildFailed(std::io::Error::other(
            PanicOnDisplay,
        )))
    }
}

/// Error whose `Display` and `Drop` both panic.
///
/// Formatting one of these errors unwinds, and disposing of it afterwards
/// unwinds again. Dropping it while the first panic unwinds would abort the
/// process, so the harness must treat the two steps separately.
pub struct PanicOnDisplayAndDrop;

impl fmt::Display for PanicOnDisplayAndDrop {
    fn fmt(&self, _formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        panic_any("combined display boom")
    }
}

impl fmt::Debug for PanicOnDisplayAndDrop {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PanicOnDisplayAndDrop")
    }
}

impl std::error::Error for PanicOnDisplayAndDrop {}

impl Drop for PanicOnDisplayAndDrop {
    fn drop(&mut self) { panic_any("combined drop boom"); }
}

/// Profile whose cleanup fails with an error that panics when displayed and
/// dropped.
pub enum CleanupFailsWithHostileDisplayAndDrop {}

impl BevyProfile for CleanupFailsWithHostileDisplayAndDrop {
    fn configure(app: &mut App, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CleanupCalls>();
        app.init_resource::<RunnerCalls>();
        Ok(())
    }

    fn cleanup(scenario: &BevyScenario, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
        scenario.with_world(|world| {
            world.resource_mut::<CleanupCalls>().0 += 1;
        });
        Err(HarnessError::RuntimeBuildFailed(std::io::Error::other(
            PanicOnDisplayAndDrop,
        )))
    }
}

/// Profile whose cleanup fails with an error that panics when dropped.
pub enum CleanupFailsWithHostileDrop {}

impl BevyProfile for CleanupFailsWithHostileDrop {
    fn configure(app: &mut App, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CleanupCalls>();
        app.init_resource::<RunnerCalls>();
        Ok(())
    }

    fn cleanup(scenario: &BevyScenario, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
        scenario.with_world(|world| {
            world.resource_mut::<CleanupCalls>().0 += 1;
        });
        Err(HarnessError::RuntimeBuildFailed(std::io::Error::other(
            PanicOnErrorDrop,
        )))
    }
}

/// Plugin whose finalization hooks must never run under the harness.
pub struct PanickingFinalizePlugin;

impl Plugin for PanickingFinalizePlugin {
    fn build(&self, _app: &mut App) {}

    fn finish(&self, _app: &mut App) {
        panic!("tripwire: the harness called Plugin::finish");
    }

    fn cleanup(&self, _app: &mut App) {
        panic!("tripwire: the harness called Plugin::cleanup");
    }
}

/// Profile that installs a plugin with panicking finalization hooks.
pub enum FinalizeProbe {}

impl BevyProfile for FinalizeProbe {
    fn configure(app: &mut App, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
        app.add_plugins((MinimalPlugins, PanickingFinalizePlugin));
        Ok(())
    }
}

/// Profile that fixes the time step so each update runs one fixed update.
pub enum FixedTimestep {}

impl BevyProfile for FixedTimestep {
    fn configure(app: &mut App, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
        app.init_resource::<FixedTicks>();
        app.add_systems(FixedUpdate, count_fixed_tick);
        Ok(())
    }
}

fn count_fixed_tick(mut ticks: ResMut<FixedTicks>) { ticks.0 += 1; }

/// Profile that is deliberately neither `Send` nor `Sync`.
pub struct NotSendProfile {
    _marker: Rc<()>,
}

impl BevyProfile for NotSendProfile {
    fn configure(_app: &mut App, _metadata: &ScenarioMetadata) -> HarnessResult<()> { Ok(()) }
}
