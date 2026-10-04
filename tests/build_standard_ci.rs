//! Contract test for the order of CI's gates and where it installs `mold`.
//!
//! The Makefile restates the build standard's `mold` flag for its gate targets,
//! so the job must install `mold` before `make lint` and `make test`, the steps
//! that take it. Coverage does not need it: CI's setup-rust exports `RUSTFLAGS`,
//! which displaces the configured linker flags for the coverage step, so its
//! effective flags carry no `mold` flag whether or not it links. `make test`
//! must also run after lint and before coverage, so the whole suite is exercised
//! under Cranelift before coverage swaps the backend to LLVM.
//!
//! The order is one validator, `order_problem`, applied to the real `ci.yml` and
//! to every fixture. It reads the workflow as text with the reader in
//! `build_standard/workflow.rs`, which counts only an unconditional `run`
//! command or `uses:` action: not a comment, a name, a description, an `echo`
//! of a gate, or a step with an `if:`. Tests over fixed workflows hold the reader.
//!
//! File access goes through a `cap_std` directory handle rooted at the crate
//! manifest directory.

use std::error::Error;

use cap_std::{ambient_authority, fs::Dir};
use rstest::rstest;

#[path = "build_standard/workflow.rs"]
mod workflow;

use workflow::Job;

/// The result of a reader, which the tests unwrap.
type Read<T> = Result<T, Box<dyn Error>>;

/// The shared action that measures coverage, by path.
const COVERAGE_ACTION: &str = "leynos/shared-actions/.github/actions/generate-coverage";

/// Reads a file relative to the crate manifest directory.
fn read(path: &str) -> Read<String> {
    let root = Dir::open_ambient_dir(env!("CARGO_MANIFEST_DIR"), ambient_authority())?;
    Ok(root.read_to_string(path)?)
}

/// A workflow whose coverage job installs `mold`, then lints, tests and
/// measures coverage.
const GOOD_WORKFLOW: &str = concat!(
    "jobs:\n",
    "  build-test:\n",
    "    steps:\n",
    "      - name: Install mold linker\n",
    "        run: sudo apt-get install --yes mold\n",
    "      - run: make lint\n",
    "      - run: make test\n",
    "      - name: Coverage\n",
    "        uses: leynos/shared-actions/.github/actions/generate-coverage@abc\n",
    "        env:\n",
    "          CARGO_UNSTABLE_CODEGEN_BACKEND: \"true\"\n",
    "          CARGO_PROFILE_DEV_CODEGEN_BACKEND: llvm\n",
    "        with:\n",
    "          format: lcov\n",
);

/// The installation step of [`GOOD_WORKFLOW`], which the fixtures replace.
const INSTALL_STEP: &str =
    "      - name: Install mold linker\n        run: sudo apt-get install --yes mold\n";

/// The coverage job of a workflow text.
fn coverage_job(workflow: &str) -> Job<'_> { Job::containing(workflow, "generate-coverage@") }

/// The one validator of the gate order: `mold` is installed, then `make lint`
/// and `make test` run, then coverage, each as a real unconditional step. It is
/// the check on the real workflow and on every fixture, so a fixture it rejects
/// is one the real check would reject.
fn order_problem(workflow: &str) -> Option<String> {
    let job = coverage_job(workflow);
    let steps = [
        ("the mold installation", job.mold_install_offset()),
        ("`make lint`", job.gate_offset("lint")),
        ("`make test`", job.gate_offset("test")),
        ("the coverage action", job.action_offset(COVERAGE_ACTION)),
    ];
    let mut before: Option<(&str, usize)> = None;
    for (name, found) in steps {
        let Some(at) = found else {
            return Some(format!("the coverage job lacks {name} as a real step"));
        };
        if let Some((earlier, earlier_at)) = before.filter(|&(_, earlier_at)| earlier_at >= at) {
            return Some(format!(
                "{earlier} (line {earlier_at}) is not before {name}"
            ));
        }
        before = Some((name, at));
    }
    None
}

#[test]
fn ci_runs_its_gates_in_order() {
    let workflow = read(".github/workflows/ci.yml").expect("read ci.yml");
    assert_eq!(order_problem(&workflow), None);
}

#[test]
fn the_validator_accepts_the_good_workflow() {
    assert_eq!(order_problem(GOOD_WORKFLOW), None);
}

/// Each fixture breaks the gate order in one way, and the validator that checks
/// the real workflow must reject it for that reason.
#[rstest]
#[case::no_install(
    GOOD_WORKFLOW.replace(INSTALL_STEP, ""),
    "lacks the mold installation"
)]
#[case::install_after_lint(
    GOOD_WORKFLOW.replace(INSTALL_STEP, "").replace(
        "      - run: make lint\n",
        "      - run: make lint\n      - run: sudo apt-get install mold\n",
    ),
    "is not before `make lint`"
)]
#[case::echoed_lint(
    GOOD_WORKFLOW.replace("run: make lint", "run: echo make lint"),
    "lacks `make lint`"
)]
#[case::echoed_test(
    GOOD_WORKFLOW.replace("run: make test", "run: echo make test"),
    "lacks `make test`"
)]
#[case::name_only_test(
    GOOD_WORKFLOW.replace("      - run: make test\n", "      - name: make test\n"),
    "lacks `make test`"
)]
#[case::conditional_test(
    GOOD_WORKFLOW.replace("      - run: make test\n", "      - if: false\n        run: make test\n"),
    "lacks `make test`"
)]
#[case::missing_test(
    GOOD_WORKFLOW.replace("      - run: make test\n", ""),
    "lacks `make test`"
)]
#[case::test_after_coverage(
    GOOD_WORKFLOW.replace("      - run: make test\n", "").replace(
        "          format: lcov\n",
        "          format: lcov\n      - run: make test\n",
    ),
    "is not before the coverage action"
)]
#[case::conditional_coverage(
    GOOD_WORKFLOW.replace(
        "        uses: leynos/shared-actions/.github/actions/generate-coverage@abc\n",
        "        if: false\n        uses: leynos/shared-actions/.github/actions/generate-coverage@abc\n",
    ),
    "lacks the coverage action"
)]
fn the_validator_rejects_a_broken_order(#[case] workflow: String, #[case] problem: &str) {
    let found = order_problem(&workflow).expect("the validator accepted a broken order");
    assert!(found.contains(problem), "{found}");
}

#[rstest]
#[case::real_install("      - run: sudo apt-get install --yes mold\n", true)]
#[case::inert_echo("      - run: echo sudo apt-get install mold\n", false)]
#[case::step_first_if(
    "      - if: false\n        run: sudo apt-get install --yes mold\n",
    false
)]
#[case::later_if(
    "      - run: sudo apt-get install --yes mold\n        if: false\n",
    false
)]
#[case::quoted_echo("      - run: echo \"x && sudo apt-get install mold\"\n", false)]
#[case::description_text("      - description: sudo apt-get install mold\n", false)]
#[case::block_run(
    "      - run: |\n          sudo apt-get update\n          sudo apt-get install mold\n",
    true
)]
#[case::folded_echo(
    "      - run: >\n          echo skipped\n          sudo apt-get install mold\n",
    false
)]
#[case::folded_install(
    "      - run: >\n          sudo apt-get install\n          --yes mold\n",
    true
)]
#[case::setup_rust_input(
    "      - uses: leynos/shared-actions/.github/actions/setup-rust@abc\n        with:\n          \
     install-mold: true\n",
    true
)]
#[case::input_false_with_comment(
    "      - uses: leynos/shared-actions/.github/actions/setup-rust@abc\n        with:\n          \
     install-mold: false # true\n",
    false
)]
#[case::input_not_exactly_true(
    "      - uses: leynos/shared-actions/.github/actions/setup-rust@abc\n        with:\n          \
     install-mold: untrue\n",
    false
)]
#[case::input_in_env_not_with(
    "      - uses: leynos/shared-actions/.github/actions/setup-rust@abc\n        env:\n          \
     install-mold: true\n",
    false
)]
#[case::lookalike_action(
    "      - uses: org/not-setup-rust-really@abc\n        with:\n          install-mold: true\n",
    false
)]
#[case::unrelated_action_input(
    "      - uses: org/other-action@abc\n        with:\n          install-mold: true\n",
    false
)]
fn the_install_reader_counts_only_runnable_installs(#[case] step: &str, #[case] counts: bool) {
    let workflow = GOOD_WORKFLOW.replace(INSTALL_STEP, step);
    assert_eq!(
        coverage_job(&workflow).mold_install_offset().is_some(),
        counts,
        "{step}"
    );
}
