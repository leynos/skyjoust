//! Contract test for the dev-fast wiring in the repository `Makefile`.
//!
//! Sponsor decision: the dev-fast profile (the Cranelift codegen backend
//! plus the mold linker, configured in `tools/dev-fast/config.toml`) is
//! the *standard* development path, documented in AGENTS.md under "Fast
//! development builds". The standard `build`, `test`, `lint`, and
//! `typecheck` targets must pass `--config tools/dev-fast/config.toml` to
//! every `cargo` invocation they make, and the `coverage` target (should
//! one exist) must never receive it, since coverage instrumentation needs
//! the supported LLVM backend and platform linker.
//!
//! This test reads the checked-in `Makefile` at a compile-time-known path
//! and asserts on its literal recipe text, so an agent editing the
//! Makefile without preserving the wiring fails this suite locally,
//! before the estate audit (concordat's DF-004 rule) ever runs.

/// The checked-in `Makefile`, embedded at compile time from the crate
/// root (this test lives in the root `skyjoust` package, whose
/// `CARGO_MANIFEST_DIR` is the repository root).
const MAKEFILE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Makefile"));

/// Path, relative to the repository root, of the dev-fast Cargo
/// configuration fragment the wired targets pass via `--config`.
const DEV_FAST_CONFIG_PATH: &str = "tools/dev-fast/config.toml";

/// One standard development target and the name of the Makefile rule
/// whose recipe text actually carries the dev-fast wiring.
///
/// Usually identical to `target`. `build` is the one exception: it has
/// no recipe of its own, only a prerequisite on the debug binary, which
/// the `target/%/$(TARGET)` pattern rule builds for both `build` and
/// `release`. The dev-fast `--config` flag was added to that pattern
/// rule's debug branch, so `build`'s wiring is asserted there.
struct StandardTarget {
    /// The `make <target>` name a contributor or CI invokes.
    target: &'static str,
    /// The Makefile rule whose recipe text is checked for the wiring.
    recipe_rule: &'static str,
}

/// The standard development targets that must pass `--config
/// tools/dev-fast/config.toml` to every `cargo` invocation they make.
const STANDARD_TARGETS: &[StandardTarget] = &[
    StandardTarget {
        target: "build",
        recipe_rule: "target/%/$(TARGET)",
    },
    StandardTarget {
        target: "test",
        recipe_rule: "test",
    },
    StandardTarget {
        target: "lint",
        recipe_rule: "lint",
    },
    StandardTarget {
        target: "typecheck",
        recipe_rule: "typecheck",
    },
];

/// Extract a Makefile rule's recipe block: the line beginning with
/// `{rule}:`, followed by every subsequent line that starts with a tab.
/// Returns `None` when the rule is not defined at all, matching this
/// test's "where the target exists" scoping.
fn recipe_block(makefile: &str, rule: &str) -> Option<String> {
    let marker = format!("{rule}:");
    let lines: Vec<&str> = makefile.lines().collect();
    let start = lines.iter().position(|line| line.starts_with(&marker))?;
    let block: Vec<&str> = std::iter::once(*lines.get(start)?)
        .chain(
            lines
                .iter()
                .skip(start + 1)
                .copied()
                .take_while(|line| line.starts_with('\t')),
        )
        .collect();
    Some(block.join("\n"))
}

/// Case-insensitively check `text` for a `dev-fast`/`dev_fast` mention,
/// matching the `(?i)dev[-_]fast` convention this contract enforces.
fn mentions_dev_fast(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("dev-fast") || lower.contains("dev_fast")
}

#[rstest::rstest]
#[case::make_build("build")]
#[case::make_test("test")]
#[case::make_lint("lint")]
#[case::make_typecheck("typecheck")]
fn standard_target_recipe_wires_dev_fast_config(#[case] target: &str) {
    let standard_target = STANDARD_TARGETS
        .iter()
        .find(|entry| entry.target == target)
        .expect("target must be listed in STANDARD_TARGETS");
    let recipe = recipe_block(MAKEFILE, standard_target.recipe_rule).unwrap_or_else(|| {
        panic!(
            "Makefile rule {:?} (backing `make {target}`) not found; the dev-fast standard-path \
             convention documented in AGENTS.md under \"Fast development builds\" requires every \
             standard build/test/lint/typecheck target to exist and pass `--config \
             {DEV_FAST_CONFIG_PATH}` to cargo",
            standard_target.recipe_rule
        )
    });

    assert!(
        recipe.contains("--config"),
        "`make {target}`'s recipe (rule {:?}) must pass `--config` to every cargo invocation, per \
         the dev-fast standard-path convention documented in AGENTS.md under \"Fast development \
         builds\"; recipe was:\n{recipe}",
        standard_target.recipe_rule
    );
    assert!(
        mentions_dev_fast(&recipe),
        "`make {target}`'s recipe (rule {:?}) must reference the dev-fast config fragment \
         (matching `dev-fast`/`dev_fast`), per the convention documented in AGENTS.md under \
         \"Fast development builds\"; recipe was:\n{recipe}",
        standard_target.recipe_rule
    );
}

#[test]
fn coverage_target_recipe_excludes_dev_fast_config() {
    let Some(recipe) = recipe_block(MAKEFILE, "coverage") else {
        // No `coverage` target in this Makefile's own build/test/lint
        // targets; coverage runs through a separate CI action instead.
        // Nothing to assert.
        return;
    };

    assert!(
        !mentions_dev_fast(&recipe),
        "the `coverage` target's recipe must never reference the dev-fast config fragment: \
         coverage instrumentation needs the supported LLVM backend and platform linker, per \
         AGENTS.md's \"Fast development builds\" section; recipe was:\n{recipe}"
    );
}

#[test]
fn dev_fast_config_fragment_exists() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/",
        "tools/dev-fast/config.toml"
    );

    assert!(
        std::path::Path::new(path).is_file(),
        "expected the dev-fast Cargo configuration fragment at {DEV_FAST_CONFIG_PATH}, referenced \
         by the standard build/test/lint/typecheck targets and documented in AGENTS.md under \
         \"Fast development builds\""
    );
}
