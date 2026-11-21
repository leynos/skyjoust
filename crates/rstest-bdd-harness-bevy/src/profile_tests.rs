//! Unit tests for the headless plugin configuration.

use bevy::{
    app::App,
    diagnostic::FrameCount,
    ecs::resource::Resource,
    prelude::Update,
    time::TimePlugin,
};
use googletest::prelude::*;
use rstest::rstest;
use rstest_bdd_harness::ScenarioMetadata;

use super::{BareBevyProfile, BevyProfile, MinimalBevyProfile, add_minimal_plugins, minimal_app};

/// Configures a fresh application with `Profile` and asserts the hook succeeds.
fn configured_app<Profile: BevyProfile>() -> App {
    let mut app = App::new();
    let result = Profile::configure(&mut app, &ScenarioMetadata::default());
    assert!(result.is_ok(), "the profile's configure hook must succeed");
    app
}

#[gtest]
#[rstest]
#[case(0)]
#[case(3)]
fn minimal_app_counts_frames(#[case] ticks: u32) {
    let mut app = minimal_app();
    (0..ticks).for_each(|_| app.update());
    expect_that!(app.world().resource::<FrameCount>().0, eq(ticks));
}

#[gtest]
#[rstest]
fn minimal_app_adds_the_time_plugin() {
    let app = minimal_app();
    expect_that!(app.is_plugin_added::<TimePlugin>(), eq(true));
}

#[gtest]
#[rstest]
fn an_unconfigured_app_omits_the_time_plugin() {
    let app = App::new();
    expect_that!(app.is_plugin_added::<TimePlugin>(), eq(false));
}

#[gtest]
fn bare_profile_leaves_the_app_unconfigured() {
    let app = configured_app::<BareBevyProfile>();
    expect_that!(app.is_plugin_added::<TimePlugin>(), eq(false));
    expect_that!(
        app.world().get_resource::<FrameCount>().is_some(),
        eq(false)
    );
}

#[gtest]
fn bare_profile_still_runs_update_systems() {
    #[derive(Resource)]
    struct Ticks(u32);

    fn count_tick(mut ticks: bevy::ecs::system::ResMut<Ticks>) { ticks.0 += 1; }

    let mut app = configured_app::<BareBevyProfile>();
    app.insert_resource(Ticks(0));
    app.add_systems(Update, count_tick);
    app.update();
    expect_that!(app.world().resource::<Ticks>().0, eq(1));
}

#[gtest]
fn minimal_profile_adds_the_minimal_plugins() {
    let app = configured_app::<MinimalBevyProfile>();
    expect_that!(app.is_plugin_added::<TimePlugin>(), eq(true));
    expect_that!(app.world().get_resource::<FrameCount>().is_some(), eq(true));
}

#[gtest]
#[rstest]
fn add_minimal_plugins_matches_minimal_app() {
    let mut configured = App::new();
    add_minimal_plugins(&mut configured);
    let mut constructed = minimal_app();
    configured.update();
    constructed.update();
    expect_that!(configured.is_plugin_added::<TimePlugin>(), eq(true));
    expect_that!(constructed.is_plugin_added::<TimePlugin>(), eq(true));
    expect_that!(
        configured.world().resource::<FrameCount>().0,
        eq(constructed.world().resource::<FrameCount>().0)
    );
}
