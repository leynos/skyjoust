//! Unit tests for the headless plugin configuration.

use bevy::{app::App, diagnostic::FrameCount, time::TimePlugin};
use googletest::prelude::*;
use rstest::rstest;

use super::{add_minimal_plugins, minimal_app};

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
