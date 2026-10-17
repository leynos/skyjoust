# Adopt the Bevy BDD helpers in Skyjoust

This note covers the in-tree helper scaffold for Skyjoust tests. It is not a
published dependency or a runtime requirement.

## Add the test dependency

For BDD tests in the root `skyjoust` package, keep the harness and its test
framework dependencies in `[dev-dependencies]`:

```toml
[dev-dependencies]
rstest = "0.27"
rstest-bdd = "0.6.0"
rstest-bdd-macros = "0.6.0"
rstest-bdd-harness-bevy = { path = "crates/rstest-bdd-harness-bevy" }
```

Use a path relative to the consuming package for other workspace members. Do
not add the harness to `[dependencies]`: it is workspace-only and declares
`publish = false`. External consumers should wait until a separately approved
extraction and release.

## Import the headless helpers

The crate supports Bevy `0.19.1`. Bevy appears in its public API, so keep
consumers on that workspace-compatible line. The crate disables Bevy's default
features and enables `std`; its minimal profile avoids window, renderer, asset,
and audio plugins.

Use `minimal_app` for a fresh application, or `add_minimal_plugins` when a test
already owns an `App`. The test controls schedule advancement with explicit
`app.update()` calls:

```rust
use rstest_bdd_harness_bevy::{
    add_minimal_plugins,
    bevy::{app::App, diagnostic::FrameCount},
    minimal_app,
};

let mut configured = App::new();
add_minimal_plugins(&mut configured);
configured.update();

let mut scenario_app = minimal_app();
scenario_app.update();
assert_eq!(scenario_app.world().resource::<FrameCount>().0, 1);
```

Import Bevy types through the crate's `bevy` re-export when that helps keep
tests on the harness's compatibility line. Use scenario and step attributes from
`rstest-bdd-macros` and `StepResult` from `rstest-bdd`; the imported
[v0.6.0 user's guide](rstest-bdd-v0.6.0-users-guide.md) covers their usage.
This milestone supplies headless profile helpers, not the later `BevyScenario`
or profile-adapter API assigned to roadmap task `0.1.2`.
