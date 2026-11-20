//! Test profiles that record lifecycle hooks and trip on unexpected calls.
//!
//! Every profile is an uninhabited unit-like enum because `BevyProfile`'s
//! hooks are associated functions: the harness never constructs a profile
//! value. Tripwire hooks use `std::panic::panic_any` rather than `panic!`
//! because Clippy's `panic_in_result_fn` forbids panic macros inside a
//! `Result`-returning hook.

use std::{panic::panic_any, rc::Rc};

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

/// Profile whose cleanup increments the counter and then fails.
pub enum CleanupFails {}

impl BevyProfile for CleanupFails {
    fn configure(app: &mut App, _metadata: &ScenarioMetadata) -> HarnessResult<()> {
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CleanupCalls>();
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
