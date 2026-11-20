//! Inventory the workspace's `trybuild` runners against the UI binary policy.
//!
//! The Nextest policy gives the serialized compile budget to binaries matching
//! `binary(/_ui$/)`. A `trybuild` runner named otherwise would compile under the
//! ordinary 180-second limit, so every runner must carry the `_ui` suffix, and
//! the fixed inventory keeps the global run budget sized for the runners that
//! exist.

use std::io;

use cap_std::{
    ambient_authority,
    fs::{Dir, DirEntry},
};
use toml::Value;

#[test]
fn workspace_trybuild_runners_match_the_ui_binary_policy() {
    let workspace = Dir::open_ambient_dir(env!("CARGO_MANIFEST_DIR"), ambient_authority())
        .expect("workspace root must be readable");
    let manifest: Value = toml::from_str(include_str!("../../Cargo.toml"))
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
        if let Some(runner) = trybuild_runner_name(tests, &entry?)? {
            runners.push(runner);
        }
    }
    Ok(runners)
}

/// Name the entry when it is a Rust source file that drives `trybuild`.
///
/// Only Rust files are read, so unrelated fixtures beside the runners are
/// never required to be UTF-8 text.
fn trybuild_runner_name(tests: &Dir, entry: &DirEntry) -> io::Result<Option<String>> {
    if !entry.file_type()?.is_file() {
        return Ok(None);
    }
    let filename = entry.file_name();
    let filename_text = filename
        .to_str()
        .ok_or_else(|| io::Error::other("test filenames must be UTF-8"))?;
    if !has_rust_extension(filename_text) {
        return Ok(None);
    }
    let source = tests.read_to_string(&filename)?;
    // The needle is assembled so this file does not match its own search.
    let is_trybuild_runner = source.contains(concat!("trybuild::", "TestCases"));
    Ok(is_trybuild_runner.then(|| filename_text.to_owned()))
}

fn has_rust_extension(filename: &str) -> bool {
    filename
        .rsplit_once('.')
        .is_some_and(|(_, extension)| extension.eq_ignore_ascii_case("rs"))
}
