//! End-to-end tests for the `validate_trace` command-line interface.

use std::{
    error::Error,
    io::Write,
    process::{Command, Output, Stdio},
};

use serde_json::Value;

#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "assert! gives a clearer failure message than manually building an Err here"
)]
fn valid_trace_prints_pretty_json_and_exits_zero() -> Result<(), Box<dyn Error>> {
    let output = run_validate_trace(
        &[],
        include_str!("../traces/keep_breach_reward_commit.json"),
    )?;

    assert!(output.status.success());
    assert!(stderr_text(&output)?.is_empty());
    let stdout = stdout_text(&output)?;
    let json: Value = serde_json::from_str(stdout)?;
    assert_eq!(field(&json, "ok")?, &Value::Bool(true));
    assert!(field(&json, "final_state")?.is_object());
    assert!(field(&json, "failure")?.is_null());
    Ok(())
}

#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "assert! gives a clearer failure message than manually building an Err here"
)]
fn verbose_trace_prints_replay_diagnostics_to_stderr() -> Result<(), Box<dyn Error>> {
    let output = run_validate_trace(
        &["--verbose"],
        include_str!("../traces/keep_breach_reward_commit.json"),
    )?;

    assert!(output.status.success());
    let stderr = stderr_text(&output)?;
    assert!(stderr.contains("trace step 0: AssetsLoaded"));
    assert!(stderr.contains("trace final state: ok=true"));
    let json: Value = serde_json::from_str(stdout_text(&output)?)?;
    assert_eq!(field(&json, "ok")?, &Value::Bool(true));
    Ok(())
}

#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "assert! gives a clearer failure message than manually building an Err here"
)]
fn invalid_trace_exits_two_and_prints_failure_json() -> Result<(), Box<dyn Error>> {
    let output = run_validate_trace(&[], r#"["AssetsLoaded","StartSkirmish","CommitRewards"]"#)?;

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr_text(&output)?.is_empty());
    let json: Value = serde_json::from_str(stdout_text(&output)?)?;
    assert_eq!(field(&json, "ok")?, &Value::Bool(false));
    let final_state = field(&json, "final_state")?;
    assert!(final_state.is_object());
    let failure = field(&json, "failure")?;
    assert!(failure.is_object());
    assert_eq!(field(failure, "step_index")?, &Value::from(2));
    assert!(
        field(failure, "reason")?
            .as_str()
            .is_some_and(|reason| reason.contains("action was not legal"))
    );
    Ok(())
}

fn run_validate_trace(args: &[&str], input: &str) -> Result<Output, Box<dyn Error>> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_validate_trace"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdin = child
        .stdin
        .as_mut()
        .ok_or("validate_trace child should expose stdin")?;
    stdin.write_all(input.as_bytes())?;

    Ok(child.wait_with_output()?)
}

fn stdout_text(output: &Output) -> Result<&str, Box<dyn Error>> {
    Ok(std::str::from_utf8(&output.stdout)?)
}

fn stderr_text(output: &Output) -> Result<&str, Box<dyn Error>> {
    Ok(std::str::from_utf8(&output.stderr)?)
}

/// Look up a required JSON object field, returning an error rather than
/// panicking when the field is absent.
fn field<'a>(value: &'a Value, key: &str) -> Result<&'a Value, Box<dyn Error>> {
    value
        .get(key)
        .ok_or_else(|| format!("expected JSON field {key:?}").into())
}
