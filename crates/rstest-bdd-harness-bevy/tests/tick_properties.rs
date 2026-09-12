//! Property coverage for the bounded tick invariant: after `n` calls to
//! `App::update`, the frame count is `n`.

use bevy::diagnostic::FrameCount;
use proptest::prelude::*;
use rstest_bdd_harness_bevy::minimal_app;

proptest! {
    #![proptest_config(ProptestConfig { cases: 32, ..ProptestConfig::default() })]

    #[test]
    fn frame_count_tracks_update_calls(ticks in 0_u32..=32) {
        let mut app = minimal_app();
        (0..ticks).for_each(|_| app.update());
        prop_assert_eq!(app.world().resource::<FrameCount>().0, ticks);
    }
}
