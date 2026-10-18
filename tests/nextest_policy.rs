//! Guard the bounded Nextest policy for compile-heavy UI test runners.

use std::io;

use cap_std::{ambient_authority, fs::Dir};
use rstest::{fixture, rstest};
use toml::Value;

// GitHub-hosted jobs terminate after six hours:
// https://docs.github.com/en/enterprise-cloud@latest/actions/reference/limits
const GITHUB_HOSTED_JOB_TIMEOUT_MINUTES: u64 = 360;
const COVERAGE_ACTION_PREFIX: &str = "leynos/shared-actions/.github/actions/generate-coverage@";

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
        Some("190m"),
        "the run limit must contain both serialized UI runners"
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
        Some(90),
        "UI tests must terminate after ninety minutes rather than run indefinitely"
    );
    assert_eq!(
        timeout.get("grace-period").and_then(Value::as_str),
        Some("5s"),
        "UI tests must retain a bounded termination grace period"
    );

    let run_minutes = config
        .get("profile")
        .and_then(|profiles| profiles.get("default"))
        .and_then(|defaults| defaults.get("global-timeout"))
        .and_then(Value::as_str)
        .and_then(|duration| duration.strip_suffix('m'))
        .and_then(|minutes| minutes.parse::<i64>().ok())
        .expect("the global run budget must be expressed in whole minutes");
    let compile_periods = timeout
        .get("terminate-after")
        .and_then(Value::as_integer)
        .expect("the UI compile budget must be finite");
    // The inventory test below fixes two UI runners. Include both grace
    // periods and one full ordinary-test tail after the serialized group.
    assert!(
        run_minutes * 60 > 2 * (compile_periods * 60 + 5) + 180 + 5,
        "the global budget must contain both UI runners and an ordinary-test tail"
    );
}

#[rstest]
#[case(include_str!("../.github/workflows/ci.yml"), "coverage-ratchet")]
#[case(include_str!("../.github/workflows/coverage-main.yml"), "coverage-upload")]
fn coverage_watchdogs_contain_the_test_run(
    policy: Result<Value, toml::de::Error>,
    #[case] source: &str,
    #[case] job_name: &str,
) {
    let config = policy.expect("Nextest policy must be valid TOML");
    let run_minutes = config
        .get("profile")
        .and_then(|profiles| profiles.get("default"))
        .and_then(|defaults| defaults.get("global-timeout"))
        .and_then(Value::as_str)
        .and_then(|duration| duration.strip_suffix('m'))
        .and_then(|minutes| minutes.parse::<u64>().ok())
        .expect("the global run budget must be expressed in whole minutes");
    let workflow: serde_norway::Value =
        serde_norway::from_str(source).expect("the workflow must be valid YAML");
    let steps = workflow
        .get("jobs")
        .and_then(|jobs| jobs.get(job_name))
        .and_then(|job| job.get("steps"))
        .and_then(serde_norway::Value::as_sequence)
        .expect("the coverage job must contain steps");
    let coverage = steps
        .iter()
        .find(|step| {
            step.get("uses")
                .and_then(serde_norway::Value::as_str)
                .is_some_and(|uses| uses.starts_with(COVERAGE_ACTION_PREFIX))
        })
        .expect("the workflow must invoke the coverage action");
    let cargo_seconds = coverage
        .get("with")
        .and_then(|inputs| inputs.get("cargo-wait-timeout"))
        .and_then(serde_norway::Value::as_str)
        .and_then(|seconds| seconds.parse::<u64>().ok())
        .expect("coverage must set an explicit finite Cargo watchdog");
    let step_minutes = coverage
        .get("timeout-minutes")
        .and_then(serde_norway::Value::as_u64)
        .expect("coverage must set an explicit step watchdog");

    assert!(
        cargo_seconds >= (run_minutes + 30) * 60,
        "the Cargo watchdog must allow the run budget plus thirty minutes for cold compilation"
    );
    assert!(
        step_minutes * 60 > cargo_seconds,
        "the CI step must leave time for watchdog cleanup and coverage reporting"
    );
}

#[test]
fn pull_request_coverage_has_an_independent_job_budget() {
    let workflow: serde_norway::Value =
        serde_norway::from_str(include_str!("../.github/workflows/ci.yml"))
            .expect("the CI workflow must be valid YAML");
    let jobs = workflow
        .get("jobs")
        .expect("the CI workflow must define jobs");
    let build_test = jobs
        .get("build-test")
        .expect("the CI workflow must define the build-test job");
    let coverage_job = jobs
        .get("coverage-ratchet")
        .expect("the CI workflow must define the coverage-ratchet job");
    for (name, job) in [
        ("build-test", build_test),
        ("coverage-ratchet", coverage_job),
    ] {
        let timeout = job
            .get("timeout-minutes")
            .and_then(serde_norway::Value::as_u64)
            .expect("each CI job must set an explicit finite job timeout");
        assert_eq!(
            timeout, GITHUB_HOSTED_JOB_TIMEOUT_MINUTES,
            "the {name} budget must match GitHub-hosted job limits"
        );
    }

    assert_eq!(
        coverage_job
            .get("needs")
            .and_then(serde_norway::Value::as_str),
        Some("build-test"),
        "coverage must wait for the Cranelift test job to finish"
    );

    let build_steps = build_test
        .get("steps")
        .and_then(serde_norway::Value::as_sequence)
        .expect("the build-test job must contain steps");
    assert!(
        !build_steps.iter().any(|step| {
            step.get("uses")
                .and_then(serde_norway::Value::as_str)
                .is_some_and(|uses| uses.starts_with(COVERAGE_ACTION_PREFIX))
        }),
        "coverage must not consume time from the Cranelift test job"
    );
}

#[rstest]
#[case::complete_test_target("build-test", "run", "make test")]
#[case::coverage_action("coverage-ratchet", "uses", COVERAGE_ACTION_PREFIX)]
fn coverage_step_timeouts_fit_their_job_budgets(
    #[case] job_name: &str,
    #[case] field: &str,
    #[case] expected: &str,
) {
    let workflow: serde_norway::Value =
        serde_norway::from_str(include_str!("../.github/workflows/ci.yml"))
            .expect("the CI workflow must be valid YAML");
    let job = workflow
        .get("jobs")
        .and_then(|jobs| jobs.get(job_name))
        .expect("the CI workflow must define the budgeted job");
    let job_timeout = job
        .get("timeout-minutes")
        .and_then(serde_norway::Value::as_u64)
        .expect("the CI job must set an explicit finite job timeout");
    let steps = job
        .get("steps")
        .and_then(serde_norway::Value::as_sequence)
        .expect("the CI job must contain steps");
    let step = steps
        .iter()
        .find(|step| {
            step.get(field)
                .and_then(serde_norway::Value::as_str)
                .is_some_and(|value| match field {
                    "run" => value.trim() == expected,
                    "uses" => value.starts_with(expected),
                    _ => false,
                })
        })
        .expect("the CI job must contain the budgeted step");
    let step_timeout = step
        .get("timeout-minutes")
        .and_then(serde_norway::Value::as_u64)
        .expect("the CI step must set an explicit finite step timeout");

    assert!(
        step_timeout < job_timeout,
        "the {job_name} step timeout must fit within its own job budget"
    );
}

#[rstest]
#[case(include_str!("../.github/workflows/ci.yml"), "coverage-ratchet")]
#[case(include_str!("../.github/workflows/coverage-main.yml"), "coverage-upload")]
fn coverage_jobs_prepare_dependencies_before_coverage(
    #[case] source: &str,
    #[case] job_name: &str,
) {
    let workflow: serde_norway::Value =
        serde_norway::from_str(source).expect("the coverage workflow must be valid YAML");
    let steps = workflow
        .get("jobs")
        .and_then(|jobs| jobs.get(job_name))
        .and_then(|job| job.get("steps"))
        .and_then(serde_norway::Value::as_sequence)
        .expect("the coverage job must contain steps");
    let preparation_index = steps
        .iter()
        .position(|step| {
            step.get("run")
                .and_then(serde_norway::Value::as_str)
                .is_some_and(|command| command.trim() == "make test-prepare")
        })
        .expect("coverage must prepare locked dependencies");
    let preparation = steps
        .get(preparation_index)
        .expect("the preparation step index must identify a workflow step");
    assert_eq!(
        preparation
            .get("timeout-minutes")
            .and_then(serde_norway::Value::as_u64),
        Some(15),
        "dependency preparation must keep its bounded 15-minute timeout"
    );
    let coverage_index = steps
        .iter()
        .position(|step| {
            step.get("uses")
                .and_then(serde_norway::Value::as_str)
                .is_some_and(|uses| uses.starts_with(COVERAGE_ACTION_PREFIX))
        })
        .expect("coverage workflow must invoke the coverage action");
    assert!(
        preparation_index < coverage_index,
        "coverage must prepare dependencies before its timed action starts"
    );
}

#[rstest]
fn test_step_watchdog_contains_nextest_and_doctests(policy: Result<Value, toml::de::Error>) {
    let config = policy.expect("Nextest policy must be valid TOML");
    let run_minutes = config
        .get("profile")
        .and_then(|profiles| profiles.get("default"))
        .and_then(|defaults| defaults.get("global-timeout"))
        .and_then(Value::as_str)
        .and_then(|duration| duration.strip_suffix('m'))
        .and_then(|minutes| minutes.parse::<u64>().ok())
        .expect("the global run budget must be expressed in whole minutes");
    let workflow: serde_norway::Value =
        serde_norway::from_str(include_str!("../.github/workflows/ci.yml"))
            .expect("the CI workflow must be valid YAML");
    let steps = workflow
        .get("jobs")
        .and_then(|jobs| jobs.get("build-test"))
        .and_then(|job| job.get("steps"))
        .and_then(serde_norway::Value::as_sequence)
        .expect("the CI job must contain steps");
    let test = steps
        .iter()
        .find(|step| {
            step.get("run")
                .and_then(serde_norway::Value::as_str)
                .is_some_and(|command| command.trim() == "make test")
        })
        .expect("the CI job must run the complete test target");
    let step_minutes = test
        .get("timeout-minutes")
        .and_then(serde_norway::Value::as_u64)
        .expect("the test target must have an explicit step watchdog");

    assert!(
        step_minutes > run_minutes + 30,
        "the CI step must leave thirty minutes beyond Nextest for compilation and doctests"
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
