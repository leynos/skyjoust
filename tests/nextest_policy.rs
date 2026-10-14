//! Guard the bounded Nextest policy for compile-heavy UI test runners.

use std::io;

use cap_std::{ambient_authority, fs::Dir};
use rstest::{fixture, rstest};
use toml::Value;

#[fixture]
fn policy() -> Result<Value, toml::de::Error> {
    toml::from_str(include_str!("../.config/nextest.toml"))
}

#[rstest]
fn ordinary_tests_keep_the_shared_action_timeouts(policy: Result<Value, toml::de::Error>) {
    let config = policy.expect("Nextest policy must be valid TOML");
    let defaults = config
        .get("profile")
        .and_then(|profiles| profiles.get("default"))
        .expect("Nextest policy must define the default profile");
    let timeout = defaults
        .get("slow-timeout")
        .expect("ordinary tests must retain a finite slow timeout");

    assert_eq!(
        defaults.get("global-timeout").and_then(Value::as_str),
        Some("10m"),
        "the shared action's global run limit must remain unchanged"
    );
    assert_eq!(
        timeout.get("period").and_then(Value::as_str),
        Some("180s"),
        "ordinary tests must retain the shared action's 180-second limit"
    );
    assert_eq!(
        timeout.get("terminate-after").and_then(Value::as_integer),
        Some(1),
        "ordinary tests must terminate after one timeout period"
    );
    assert_eq!(
        timeout.get("grace-period").and_then(Value::as_str),
        Some("5s"),
        "ordinary tests must retain the shared action's termination grace period"
    );
}

#[rstest]
fn only_ui_binaries_receive_the_serial_compile_budget(policy: Result<Value, toml::de::Error>) {
    let config = policy.expect("Nextest policy must be valid TOML");
    let group = config
        .get("test-groups")
        .and_then(|groups| groups.get("compile-ui"))
        .expect("compile-heavy UI tests must have a dedicated test group");
    assert_eq!(
        group.get("max-threads").and_then(Value::as_integer),
        Some(1),
        "only one compile-heavy UI test may run at a time"
    );

    let overrides = config
        .get("profile")
        .and_then(|profiles| profiles.get("default"))
        .and_then(|defaults| defaults.get("overrides"))
        .and_then(Value::as_array)
        .expect("the default profile must contain the UI override");
    assert_eq!(
        overrides.len(),
        1,
        "the UI policy must not alter unrelated tests"
    );
    let ui = overrides.first().expect("the UI override must exist");
    assert_eq!(
        ui.get("filter").and_then(Value::as_str),
        Some("binary(/_ui$/)"),
        "the compile budget must be scoped to UI binary names"
    );
    assert_eq!(
        ui.get("test-group").and_then(Value::as_str),
        Some("compile-ui"),
        "UI binaries must share the serialized compile group"
    );
    let timeout = ui
        .get("slow-timeout")
        .expect("UI tests must retain a finite timeout");
    assert_eq!(
        timeout.get("period").and_then(Value::as_str),
        Some("60s"),
        "UI tests must report progress every minute"
    );
    assert_eq!(
        timeout.get("terminate-after").and_then(Value::as_integer),
        Some(10),
        "UI tests must terminate after ten minutes rather than run indefinitely"
    );
    assert_eq!(
        timeout.get("grace-period").and_then(Value::as_str),
        Some("5s"),
        "UI tests must retain a bounded termination grace period"
    );
}

#[test]
fn workspace_trybuild_runners_match_the_ui_binary_policy() {
    let workspace = Dir::open_ambient_dir(env!("CARGO_MANIFEST_DIR"), ambient_authority())
        .expect("workspace root must be readable");
    let manifest: Value = toml::from_str(include_str!("../Cargo.toml"))
        .expect("workspace manifest must be valid TOML");
    let members = manifest
        .get("workspace")
        .and_then(|value| value.get("members"))
        .and_then(Value::as_array)
        .expect("workspace must list its member crates");
    let mut runners = Vec::new();

    for member in members {
        let name = member
            .as_str()
            .expect("workspace member paths must be strings");
        let tests_path = format!("{name}/tests");
        if !workspace.exists(&tests_path) {
            continue;
        }
        let tests = workspace
            .open_dir(&tests_path)
            .expect("member tests must be readable");
        for filename in trybuild_runners_in(&tests).expect("trybuild runners must be readable") {
            assert!(
                filename.ends_with("_ui.rs"),
                "trybuild runner {tests_path}/{filename} must match binary(/_ui$/)"
            );
            runners.push(format!("{tests_path}/{filename}"));
        }
    }

    runners.sort();
    assert_eq!(
        runners,
        [
            "crates/rstest-bdd-harness-bevy/tests/macro_ui.rs",
            "crates/skyjoust_test_macros/tests/fixture_expansion_lints_ui.rs",
        ],
        "keep the compile-heavy runner inventory synchronized when adding or renaming UI suites"
    );
}

// This local query only inventories direct integration targets, not UI fixtures.
fn trybuild_runners_in(tests: &Dir) -> io::Result<Vec<String>> {
    let mut runners = Vec::new();
    for entry in tests.entries()? {
        let directory_entry = entry?;
        if !directory_entry.file_type()?.is_file() {
            continue;
        }
        let filename = directory_entry.file_name();
        let filename_text = filename
            .to_str()
            .ok_or_else(|| io::Error::other("test filenames must be UTF-8"))?;
        if filename_text
            .rsplit_once('.')
            .is_some_and(|(_, extension)| extension.eq_ignore_ascii_case("rs"))
            && tests
                .read_to_string(&filename)?
                .contains(concat!("trybuild::", "TestCases"))
        {
            runners.push(filename_text.to_owned());
        }
    }
    Ok(runners)
}
