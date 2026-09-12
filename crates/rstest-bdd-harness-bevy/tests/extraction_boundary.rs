//! Tripwire for the extraction contract: no game crate may be declared here.
//!
//! The check is textual and direct-only. It sees neither transitive edges nor
//! renamed packages; `cargo tree -p rstest-bdd-harness-bevy -e normal,dev`
//! remains the authority for the constraint as a whole.

use pretty_assertions::assert_eq;

/// Crate names this harness must never declare, per the extraction contract.
const FORBIDDEN_CRATES: [&str; 3] = ["skyjoust", "skyjoust-stateright-validator", "lille"];

/// Returns the entries of [`FORBIDDEN_CRATES`] declared in `manifest`'s
/// dependency tables.
///
/// The returned names carry the `'static` lifetime deliberately: they come
/// from the fixed list, not from `manifest`. The scan covers the dependency,
/// development-dependency, and build-dependency tables only, so a `repository`
/// field naming the Skyjoust remote cannot false-positive; and it matches
/// dependency names exactly, so the permitted `skyjoust-test-macros` passes
/// through.
fn forbidden_dependencies(manifest: &str) -> Vec<&'static str> {
    let dependency_tables = [
        "[dependencies]",
        "[dev-dependencies]",
        "[build-dependencies]",
    ];
    let mut in_dependency_table = false;
    let mut found = Vec::new();
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_dependency_table = dependency_tables.contains(&trimmed);
            continue;
        }
        if !in_dependency_table {
            continue;
        }
        let name = trimmed.split_once('=').map_or("", |(name, _)| name.trim());
        if let Some(&forbidden) = FORBIDDEN_CRATES.iter().find(|&&entry| entry == name) {
            found.push(forbidden);
        }
    }
    found
}

#[test]
fn manifest_declares_no_game_crates() {
    assert_eq!(
        forbidden_dependencies(include_str!("../Cargo.toml")),
        Vec::<&str>::new()
    );
}

#[test]
fn guard_detects_a_directly_declared_game_crate() {
    let manifest = r#"
[package]
name = "probe"

[dependencies]
some-tooling-crate = "1.0"
skyjoust = { path = "../.." }
"#;
    assert_eq!(forbidden_dependencies(manifest), vec!["skyjoust"]);
}

#[test]
fn guard_ignores_game_names_outwith_dependency_tables() {
    let manifest = r#"
[package]
name = "probe"
repository = "https://github.com/example/skyjoust"

[dependencies]
some-tooling-crate = "1.0"

[dev-dependencies.lille]
version = "0.1"
"#;
    assert_eq!(forbidden_dependencies(manifest), Vec::<&str>::new());
}

#[test]
fn guard_permits_the_test_macro_crate() {
    let manifest = r#"
[package]
name = "probe"

[dev-dependencies]
skyjoust-test-macros = { path = "../skyjoust_test_macros" }
"#;
    assert_eq!(forbidden_dependencies(manifest), Vec::<&str>::new());
}
