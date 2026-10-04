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
    lint_with_script(scratch, &script)
}

/// Runs `make lint` with `script` installed as the fake `whitaker`, returning
/// the run and the record the script wrote.
///
/// # Errors
///
/// A script that writes no record is an error carrying the run's status and
/// both output streams, so a failure to run the fake is never read as an empty
/// record.
#[cfg(unix)]
fn lint_with_script(scratch: &str, script: &str) -> Read<(Output, String)> {
    use cap_std::fs::{OpenOptions, OpenOptionsExt};

    // The tool directory is rebuilt clean, so a record from an earlier run
    // cannot survive, and it holds the only tools on `PATH`.
    let root = tool_directory(scratch)?;
    let dir = Dir::open_ambient_dir(&root, ambient_authority())?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true).mode(0o755);
    std::io::Write::write_all(&mut dir.open_with("whitaker", &options)?, script.as_bytes())?;
    // A competing `whitaker` in the home directory the Makefile prepends to
    // `PATH`: if the fake were found by name it could be shadowed by this one.
    dir.create_dir_all(".cargo/bin")?;
    let mut competing = OpenOptions::new();
    competing.write(true).create_new(true).mode(0o755);
    std::io::Write::write_all(
        &mut dir.open_with(".cargo/bin/whitaker", &competing)?,
        b"#!/bin/sh\necho competing > \"$(dirname \"$0\")/../../competing\"\nexit 0\n",
    )?;
    let output = Command::new("make")
        // The fake is named by its path, which beats both an inherited
        // `WHITAKER` (the bogus value below) and a competing install found
        // through `PATH`.
        .args([
            "lint".to_owned(),
            "CARGO=true".to_owned(),
            format!("WHITAKER={}", root.join("whitaker").display()),
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("PATH", &root)
        .env("WHITAKER", "/no/such/whitaker")
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
    for tool in [
        "sh", "env", "true", "make", "uname", "grep", "cat", "rm", "mktemp",
    ] {
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
        .args(["lint", "CARGO=true", "WHITAKER=whitaker"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("WHITAKER", "/no/such/whitaker")
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
    assert!(
        stdout.contains("Install whitaker"),
        "no installation guidance in: {stdout}{stderr}"
    );
    assert!(
        !tools.join("record").exists(),
        "a Whitaker run left a record although none is installed"
    );
}

/// A fake that runs but writes no record is reported with its run, not read as
/// an empty record.
#[cfg(unix)]
#[test]
fn a_fake_that_leaves_no_record_is_reported_with_its_run() {
    let message = lint_with_script("whitaker-silent", "#!/bin/sh\nexit 0\n")
        .expect_err("the fake wrote no record")
        .to_string();
    for wanted in ["left no record", "exited", "stdout", "stderr"] {
        assert!(
            message.contains(wanted),
            "`{wanted}` missing from: {message}"
        );
    }
}

/// A real `whitaker` that is also installed in the home directory the Makefile
/// searches does not shadow the fake the harness names by path: the fake's
/// record exists and the competing install never ran.
#[cfg(unix)]
#[test]
fn a_competing_home_install_does_not_shadow_the_fake() {
    let (_, record) = lint_with_fake_whitaker("whitaker-competing", 0).expect("run `make lint`");
    assert!(!record.is_empty(), "the intended fake did not run");
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("whitaker-competing");
    let dir = Dir::open_ambient_dir(&root, ambient_authority()).expect("open the scratch root");
    match dir.metadata("competing") {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Ok(_) => panic!("the competing home install ran instead of the fake"),
        Err(error) => panic!("could not check for the competing marker: {error}"),
    }
}

/// Runs `make test` with `script` as the cargo stand-in under the given test
/// runner, on a tool directory alone so nothing real can answer.
#[cfg(unix)]
fn make_test_with(scratch: &str, runner: &str, script: &str) -> Read<Output> {
    use cap_std::fs::{OpenOptions, OpenOptionsExt};

    let root = tool_directory(scratch)?;
    let dir = Dir::open_ambient_dir(&root, ambient_authority())?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true).mode(0o755);
    std::io::Write::write_all(&mut dir.open_with("cargo", &options)?, script.as_bytes())?;
    Ok(Command::new("make")
        .args([
            "test".to_owned(),
            format!("CARGO={}", root.join("cargo").display()),
            format!("TEST_CMD={runner}"),
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("PATH", &root)
        .env("HOME", &root)
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_BUILD_TARGET")
        .env_remove("MAKEFLAGS")
        .env_remove("MFLAGS")
        .env_remove("MAKELEVEL")
        .output()?)
}

/// A failing doctest fails `make test` whichever runner is selected, and the
/// recipe's one permitted skip, a package with no library target, passes.
#[cfg(unix)]
#[rstest]
#[case::nextest_doctest_fails("nextest run", "doctest failed", false)]
#[case::plain_cargo_doctest_fails("test", "doctest failed", false)]
#[case::nextest_no_library_skips("nextest run", "error: no library targets found", true)]
#[case::plain_cargo_no_library_skips("test", "error: no library targets found", true)]
fn doctest_failures_fail_make_test_under_either_runner(
    #[case] runner: &str,
    #[case] message: &str,
    #[case] passes: bool,
) {
    let script = format!(
        "#!/bin/sh\ncase \" $* \" in\n  *\" --doc \"*) echo '{message}' >&2; exit 1;;\nesac\nexit \
         0\n"
    );
    let scratch = format!("make-test-{}-{}", runner.replace(' ', "-"), passes);
    let output = make_test_with(&scratch, runner, &script).expect("run `make test`");
    assert_eq!(
        output.status.success(),
        passes,
        "`make test` under `{runner}` with `{message}` exited {}: {}{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
