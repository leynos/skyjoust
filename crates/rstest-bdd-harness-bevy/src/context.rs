//! Shared scenario handle for Bevy-backed behavioural tests.
//!
//! Steps receive a cloneable [`BevyScenario`] rather than a raw `&mut App`.
//! Clones share one application, access goes through closures so no mutable
//! borrow escapes, and a nested borrow panics with a message that names the
//! offending accessor pattern.

use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

use bevy::{
    app::App,
    ecs::{schedule::Schedules, world::World},
};
use rstest_bdd_harness::ScenarioMetadata;

/// The diagnostic for a step that borrows the application twice.
const BORROWED_MESSAGE: &str = concat!(
    "rstest-bdd-harness-bevy: BevyScenario is already borrowed; ",
    "accessor closures must not call with_app, with_world, update, or update_times"
);

/// The diagnostic for an application whose `Main` schedule was removed.
const MISSING_MAIN_MESSAGE: &str = concat!(
    "rstest-bdd-harness-bevy: the Main schedule is missing; ",
    "a system panicked during an earlier update, so this application cannot advance"
);

/// A cloneable handle to the application owned by one scenario run.
///
/// The harness gives every step a clone and keeps one for cleanup, so a step
/// cannot drop the application out from under the lifecycle. Clones share
/// state, and a clone must not outlive the scenario run.
///
/// # Examples
///
/// ```
/// use rstest_bdd_harness_bevy::BevyScenario;
///
/// fn advance(scenario: &BevyScenario) { scenario.update(); }
/// ```
#[derive(Clone, Debug)]
pub struct BevyScenario {
    app: Rc<RefCell<App>>,
    metadata: Rc<ScenarioMetadata>,
}

impl BevyScenario {
    /// Wraps a freshly configured application and its scenario metadata.
    pub(crate) fn new(app: App, metadata: ScenarioMetadata) -> Self {
        Self {
            app: Rc::new(RefCell::new(app)),
            metadata: Rc::new(metadata),
        }
    }

    /// Returns the metadata for the scenario this handle belongs to.
    ///
    /// # Returns
    ///
    /// The metadata the harness supplied for this scenario run.
    ///
    /// # Examples
    ///
    /// ```
    /// use rstest_bdd_harness_bevy::BevyScenario;
    ///
    /// fn scenario_name(scenario: &BevyScenario) -> &str { scenario.metadata().scenario_name() }
    /// ```
    #[must_use]
    pub fn metadata(&self) -> &ScenarioMetadata { &self.metadata }

    /// Runs `operation` with mutable access to the scenario application.
    ///
    /// # Parameters
    ///
    /// - `operation`: the work to run; it is called once with the application and its result is
    ///   returned unchanged.
    ///
    /// # Returns
    ///
    /// The value `operation` produces.
    ///
    /// # Panics
    ///
    /// Panics when the application is already borrowed, which happens when an
    /// accessor closure calls another accessor. Panicking is deliberate: a
    /// step must not silently skip the work it claims to perform.
    ///
    /// # Examples
    ///
    /// ```
    /// use rstest_bdd_harness_bevy::BevyScenario;
    ///
    /// fn frame_count(scenario: &BevyScenario) -> u32 {
    ///     scenario.with_app(|app| app.world().resource::<bevy::diagnostic::FrameCount>().0)
    /// }
    /// ```
    #[track_caller]
    pub fn with_app<R>(&self, operation: impl FnOnce(&mut App) -> R) -> R {
        let mut app = self.borrow_app();
        operation(&mut app)
    }

    /// Runs `operation` with mutable access to the scenario world.
    ///
    /// # Parameters
    ///
    /// - `operation`: the work to run; it is called once with the world and its result is returned
    ///   unchanged.
    ///
    /// # Returns
    ///
    /// The value `operation` produces.
    ///
    /// # Panics
    ///
    /// Panics under the same nested-borrow condition as [`Self::with_app`].
    ///
    /// # Examples
    ///
    /// ```
    /// use rstest_bdd_harness_bevy::BevyScenario;
    ///
    /// fn entity_count(scenario: &BevyScenario) -> usize {
    ///     scenario.with_world(|world| world.entities().len() as usize)
    /// }
    /// ```
    #[track_caller]
    pub fn with_world<R>(&self, operation: impl FnOnce(&mut World) -> R) -> R {
        let mut app = self.borrow_app();
        operation(app.world_mut())
    }

    /// Advances the application by exactly one frame.
    ///
    /// # Panics
    ///
    /// Panics under the same nested-borrow condition as [`Self::with_app`],
    /// and with a diagnostic when the `Main` schedule is missing because a
    /// system panicked during an earlier update.
    ///
    /// # Examples
    ///
    /// ```
    /// use rstest_bdd_harness_bevy::BevyScenario;
    ///
    /// fn advance_once(scenario: &BevyScenario) { scenario.update(); }
    /// ```
    #[track_caller]
    pub fn update(&self) { self.update_times(1); }

    /// Advances the application by `count` frames.
    ///
    /// A count of zero still checks that the application can advance, so a
    /// scenario that silently lost its `Main` schedule fails even when its
    /// steps request no ticks.
    ///
    /// # Parameters
    ///
    /// - `count`: the number of frames to advance.
    ///
    /// # Panics
    ///
    /// Panics under the same conditions as [`Self::update`].
    ///
    /// # Examples
    ///
    /// ```
    /// use rstest_bdd_harness_bevy::BevyScenario;
    ///
    /// fn advance_three(scenario: &BevyScenario) { scenario.update_times(3); }
    /// ```
    #[track_caller]
    pub fn update_times(&self, count: u32) {
        let mut app = self.borrow_app();
        ensure_main_schedule(&app);
        for _ in 0..count {
            app.update();
        }
    }

    /// Borrows the shared application or panics with the nested-borrow advice.
    ///
    /// `let ... else` is deliberate: Whitaker's `no_unwrap_or_else_panic`
    /// forbids a panicking `unwrap_or_else`, `expect` is denied in production
    /// code, and Clippy rejects both a wildcard error arm and the
    /// `if let`/`match` shapes it rewrites as `map_or_else`.
    #[track_caller]
    fn borrow_app(&self) -> RefMut<'_, App> {
        let Ok(app) = self.app.try_borrow_mut() else {
            panic!("{BORROWED_MESSAGE}");
        };
        app
    }
}

/// Panics when the application cannot run its `Main` schedule.
#[track_caller]
fn ensure_main_schedule(app: &App) {
    let has_main = app.main().update_schedule.is_some_and(|label| {
        app.world()
            .get_resource::<Schedules>()
            .is_some_and(|schedules| schedules.contains(label))
    });
    assert!(has_main, "{MISSING_MAIN_MESSAGE}");
}

#[cfg(test)]
#[path = "context_tests.rs"]
mod tests;
