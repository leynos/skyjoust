//! End-to-end tests for the validate_trace command-line interface.

use std::{
    error::Error,
    io::Write,
    process::{Command, Output, Stdio},
};

use serde_json::Value;

#[test]
fn valid_trace_prints_pretty_json_and_exits_zero() -> Result<(), Box<dyn Error>> {
    let output = run_validate_trace(
        &[],
        include_str!("../traces/keep_breach_reward_commit.json"),
    )?;

    assert!(output.status.success());
    assert!(stderr_text(&output)?.is_empty());
    let stdout = stdout_text(&output)?;
    assert!(stdout.starts_with("{\n  \"ok\": true,"));
    let json: Value = serde_json::from_str(stdout)?;
    assert_eq!(json["ok"], true);
    assert!(json["final_state"].is_object());
    assert!(json["failure"].is_null());
    Ok(())
}

#[test]
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
    assert_eq!(json["ok"], true);
    Ok(())
}

#[test]
fn invalid_trace_exits_two_and_prints_failure_json() -> Result<(), Box<dyn Error>> {
    let output = run_validate_trace(&[], r#"["AssetsLoaded","StartSkirmish","CommitRewards"]"#)?;

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr_text(&output)?.is_empty());
    let json: Value = serde_json::from_str(stdout_text(&output)?)?;
    assert_eq!(json["ok"], false);
    assert!(json["final_state"].is_object());
    assert!(json["failure"].is_object());
    assert_eq!(json["failure"]["step_index"], 2);
    assert!(
        json["failure"]["reason"]
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

    child
        .stdin
        .as_mut()
        .expect("validate_trace child should expose stdin")
        .write_all(input.as_bytes())?;

    Ok(child.wait_with_output()?)
}

fn stdout_text(output: &Output) -> Result<&str, Box<dyn Error>> {
    Ok(std::str::from_utf8(&output.stdout)?)
}

fn stderr_text(output: &Output) -> Result<&str, Box<dyn Error>> {
    Ok(std::str::from_utf8(&output.stderr)?)
}
