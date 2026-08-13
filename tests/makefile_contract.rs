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
//! before the estate audit (concordat's DF-004 rule) ever runs. Checks
//! are per cargo-invoking line, not per recipe block: a target whose
//! recipe has several `$(CARGO)` lines (nextest plus doc-tests, doc plus
//! clippy) must not pass a whole-block substring match while one of
//! those lines is unwired.

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

/// Return `true` for a line that is part of a rule's body: a tab-indented
/// recipe line, or a bare Make conditional directive (`ifeq`/`ifneq`/
/// `ifdef`/`ifndef`/`else`/`endif`). The `test` target wraps its doc-test
/// invocation in an `ifneq`/`endif` block whose directive lines sit at
/// column zero, so a recipe extractor that only follows tab-indented
/// lines would silently stop before reaching that cargo invocation.
fn is_rule_body_line(line: &str) -> bool {
    line.starts_with('\t')
        || matches!(
            line.split_whitespace().next(),
            Some("ifeq" | "ifneq" | "ifdef" | "ifndef" | "else" | "endif")
        )
}

/// Extract a Makefile rule's full body: the line beginning with
/// `{rule}:`, followed by every subsequent [`is_rule_body_line`]. Returns
/// `None` when the rule is not defined at all, matching this test's
/// "where the target exists" scoping.
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
                .take_while(|line| is_rule_body_line(line)),
        )
        .collect();
    Some(block.join("\n"))
}

/// Every line in `recipe` that invokes `$(CARGO)`. Lines that only wrap
/// cargo indirectly — such as the `lint` target's Whitaker Dylint
/// invocation, which runs under its own separately-pinned toolchain and
/// is deliberately excluded from the dev-fast fragment — are not cargo
/// lines by this definition and are correctly skipped.
fn cargo_invoking_lines(recipe: &str) -> Vec<&str> {
    recipe
        .lines()
        .filter(|line| line.contains("$(CARGO)"))
        .collect()
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
fn standard_target_cargo_lines_wire_dev_fast_config(#[case] target: &str) {
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

    let cargo_lines = cargo_invoking_lines(&recipe);
    assert!(
        !cargo_lines.is_empty(),
        "`make {target}`'s recipe (rule {:?}) has no `$(CARGO)` invocation to check; recipe \
         was:\n{recipe}",
        standard_target.recipe_rule
    );

    for cargo_line in cargo_lines {
        assert!(
            cargo_line.contains("--config"),
            "`make {target}`'s recipe (rule {:?}) has a `$(CARGO)` line that does not pass \
             `--config`, per the dev-fast standard-path convention documented in AGENTS.md under \
             \"Fast development builds\"; offending line was:\n{cargo_line}",
            standard_target.recipe_rule
        );
        assert!(
            mentions_dev_fast(cargo_line),
            "`make {target}`'s recipe (rule {:?}) has a `$(CARGO)` line that does not reference \
             the dev-fast config fragment (matching `dev-fast`/`dev_fast`), per the convention \
             documented in AGENTS.md under \"Fast development builds\"; offending line \
             was:\n{cargo_line}",
            standard_target.recipe_rule
        );
    }
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

/// Find `needle` in `haystack` at or after byte offset `after`, returning
/// an absolute offset. Uses `str::get` rather than the `[]` index
/// operator so a byte offset that lands off a UTF-8 boundary yields
/// `None` instead of panicking.
fn find_after(haystack: &str, needle: &str, after: usize) -> Option<usize> {
    haystack
        .get(after..)?
        .find(needle)
        .map(|offset| after + offset)
}

#[rstest::rstest]
#[case::make_dev_build("dev-build")]
#[case::make_dev_test("dev-test")]
fn dev_target_substitutes_cargo_via_dry_run(#[case] target: &str) {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = std::process::Command::new("make")
        .args(["--dry-run", target, "CARGO=probe-cargo"])
        .current_dir(manifest_dir)
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to run `make --dry-run {target} CARGO=probe-cargo`: {error}")
        });

    assert!(
        output.status.success(),
        "`make --dry-run {target} CARGO=probe-cargo` exited with {}; stderr was:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Each `find_after` call starts searching from the offset the previous
    // one returned, so success alone proves the three tokens appear in
    // order: the substituted cargo binary, then `--config`, then a
    // dev-fast reference.
    let cargo_index = stdout.find("probe-cargo").unwrap_or_else(|| {
        panic!(
            "`make --dry-run {target} CARGO=probe-cargo` did not substitute CARGO into the \
             emitted command, so dev-fast wiring cannot be exercised without the pinned nightly \
             toolchain or mold; emitted command was:\n{stdout}"
        )
    });
    let config_index = find_after(&stdout, "--config", cargo_index).unwrap_or_else(|| {
        panic!(
            "`make --dry-run {target} CARGO=probe-cargo`'s emitted command has no `--config` \
             after the substituted cargo binary; emitted command was:\n{stdout}"
        )
    });
    let dev_fast_index = find_after(&stdout.to_lowercase(), "dev-fast", config_index)
        .or_else(|| find_after(&stdout.to_lowercase(), "dev_fast", config_index))
        .unwrap_or_else(|| {
            panic!(
                "`make --dry-run {target} CARGO=probe-cargo`'s emitted command has no dev-fast \
                 reference after `--config`; emitted command was:\n{stdout}"
            )
        });
    assert!(
        cargo_index < config_index && config_index < dev_fast_index,
        "expected probe-cargo, --config, and a dev-fast reference in that order in `make \
         --dry-run {target} CARGO=probe-cargo`'s emitted command; emitted command was:\n{stdout}"
    );
}
