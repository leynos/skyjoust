//! Behavioural tests for the `make lint` Whitaker boundary.
//!
//! Whitaker builds its Dylint driver outside the workspace configuration, so
//! the recipe selects LLVM for it, and a failing run must fail the target while
//! a missing binary only skips the check. These tests run the real `make lint`
//! with a fake `whitaker` first on `PATH` and `true` standing in for cargo, so
//! nothing is built.
//!
//! File access goes through `cap_std` directory handles: one rooted at a
//! scratch directory under `CARGO_TARGET_TMPDIR` for the fake tool.

use std::{
    error::Error,
    process::{Command, Output},
};

use cap_std::{ambient_authority, fs::Dir};
use rstest::rstest;

/// The result of a reader, which the tests unwrap.
type Read<T> = Result<T, Box<dyn Error>>;

/// Runs `make lint` with fake tools first on `PATH`, returning whether it
/// succeeded and the environment the fake Whitaker recorded.
///
/// Cargo is replaced by `true`, so `doc` and `clippy` succeed without
/// building. The fake Whitaker writes its environment to a file and exits with
/// `whitaker_status`. Each test passes its own `scratch` directory name, since
/// the tests run concurrently and a shared directory would be cleared under one
/// of them.
#[cfg(unix)]
fn lint_with_fake_whitaker(scratch: &str, whitaker_status: i32) -> Read<(Output, String)> {
    use cap_std::fs::{OpenOptions, OpenOptionsExt};

    let target_tmp = Dir::open_ambient_dir(env!("CARGO_TARGET_TMPDIR"), ambient_authority())?;
    // A clean directory keeps a record from an earlier run out.
    target_tmp
        .remove_dir_all(scratch)
        .or_else(|error| match error.kind() {
            std::io::ErrorKind::NotFound => Ok(()),
            _ => Err(error),
        })?;
    target_tmp.create_dir(scratch)?;
    let dir = target_tmp.open_dir(scratch)?;
    // Record only the variables the tests read, so an inherited credential
    // never lands in a test artefact.
    let script = format!(
        concat!(
            "#!/bin/sh\n",
            "for name in RUSTFLAGS CARGO_UNSTABLE_CODEGEN_BACKEND \
             CARGO_PROFILE_DEV_CODEGEN_BACKEND; do\n",
            "  eval \"value=\\${{$name-__unset__}}\"\n",
            "  [ \"$value\" = __unset__ ] || echo \"$name=$value\"\n",
            "done > \"$WHITAKER_RECORD\"\n",
            "exit {}\n"
        ),
        whitaker_status
    );
    let mut options = OpenOptions::new();
    options.write(true).create_new(true).mode(0o755);
    std::io::Write::write_all(&mut dir.open_with("whitaker", &options)?, script.as_bytes())?;
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(scratch);
    // A fixed PATH keeps the run hermetic: the fake first, then the system
    // directories that hold `sh`, `env` and the `true` standing in for cargo.
    let path = format!("{}:/usr/bin:/bin", root.display());
    let output = Command::new("make")
        .args(["lint", "CARGO=true"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("PATH", path)
        .env("WHITAKER_RECORD", root.join("record"))
        // The Makefile searches `$HOME`-relative tool directories too, so point
        // it at the scratch directory rather than a real home.
        .env("HOME", &root)
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_BUILD_TARGET")
        .env_remove("MAKEFLAGS")
        .env_remove("MFLAGS")
        .env_remove("MAKELEVEL")
        .output()?;
    // A missing record means the fake never ran; surface that rather than
    // reading it as an empty successful one.
    let record = dir.read_to_string("record").map_err(|error| {
        format!(
            "the fake Whitaker left no record ({error}); `make lint` exited {} with stdout {} and \
             stderr {}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })?;
    Ok((output, record))
}

/// A Whitaker exit status decides `make lint`, and the fake is always run.
#[cfg(unix)]
#[rstest]
#[case::failing("whitaker-fails", 1)]
#[case::passing("whitaker-passes", 0)]
fn lint_succeeds_exactly_when_whitaker_does(#[case] scratch: &str, #[case] status: i32) {
    let (output, record) = lint_with_fake_whitaker(scratch, status).expect("run `make lint`");
    assert_eq!(
        output.status.success(),
        status == 0,
        "`make lint` ignored Whitaker's exit status {status}: {}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!record.is_empty(), "the fake Whitaker recorded nothing");
}

#[cfg(unix)]
#[test]
fn whitaker_builds_on_llvm_with_the_composed_flags() {
    let (output, record) = lint_with_fake_whitaker("whitaker-env", 0).expect("run `make lint`");
    assert!(
        output.status.success(),
        "`make lint` failed ({}): {}{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let value = |name: &str| {
        record
            .lines()
            .find_map(|line| line.strip_prefix(&format!("{name}=")))
            .map(str::to_owned)
    };
    assert_eq!(
        value("CARGO_UNSTABLE_CODEGEN_BACKEND").as_deref(),
        Some("true")
    );
    assert_eq!(
        value("CARGO_PROFILE_DEV_CODEGEN_BACKEND").as_deref(),
        Some("llvm")
    );
    let flags = value("RUSTFLAGS").expect("Whitaker received no RUSTFLAGS");
    assert!(
        flags.contains("-Zthreads=8"),
        "Whitaker lost the frontend flag: {flags}"
    );
}

/// Links the few system tools `make lint` needs into a scratch directory, so a
/// run whose `PATH` is only that directory cannot see an installed Whitaker.
#[cfg(unix)]
fn tool_directory(scratch: &str) -> Read<std::path::PathBuf> {
    let target_tmp = Dir::open_ambient_dir(env!("CARGO_TARGET_TMPDIR"), ambient_authority())?;
    target_tmp
        .remove_dir_all(scratch)
        .or_else(|error| match error.kind() {
            std::io::ErrorKind::NotFound => Ok(()),
            _ => Err(error),
        })?;
    target_tmp.create_dir(scratch)?;
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(scratch);
    for tool in ["sh", "env", "true", "make", "uname"] {
        let source = ["/usr/bin", "/bin"]
            .iter()
            .map(|base| std::path::Path::new(base).join(tool))
            .find(|path| path.exists())
            .ok_or_else(|| format!("no system `{tool}` to link"))?;
        // `cap_std` refuses a link to an absolute path outside its directory,
        // and a link to a system tool is exactly that.
        std::os::unix::fs::symlink(source, root.join(tool))?;
    }
    Ok(root)
}

/// A missing Whitaker binary skips the check with a message and `make lint`
/// still succeeds, since it is an optional tool; a present one that fails is
/// covered above. `PATH` is a scratch directory holding only the system tools
/// `make` needs, and `HOME` points at it, so neither an installed Whitaker nor
/// the Makefile's `$HOME`-relative search paths can supply one.
#[cfg(unix)]
#[test]
fn a_missing_whitaker_skips_the_check_and_lint_succeeds() {
    let tools = tool_directory("whitaker-absent").expect("prepare the tool directory");
    let output = Command::new("make")
        .args(["lint", "CARGO=true"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("PATH", &tools)
        .env("HOME", &tools)
        .env("WHITAKER_RECORD", tools.join("record"))
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_BUILD_TARGET")
        .env_remove("MAKEFLAGS")
        .env_remove("MFLAGS")
        .env_remove("MAKELEVEL")
        .output()
        .expect("run `make lint`");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "`make lint` failed without Whitaker ({}): {stdout}{stderr}",
        output.status
    );
    assert!(
        stdout.contains("skipping whitaker lint"),
        "no skip message in: {stdout}{stderr}"
    );
}
