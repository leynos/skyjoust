//! Validate a serialized Skyjoust action trace from standard input.
//!
//! The `main` entrypoint reads a JSON array of `SkyAction` values from stdin,
//! calls `validate_trace` with `SkyjoustInteractionModel`, prints a pretty JSON
//! `TraceValidation` result, and exits with code 2 when the trace is invalid.
//! Passing `--verbose` prints replay diagnostics to stderr without changing the
//! machine-readable output on stdout.
//!
//! `main` installs a `tracing_subscriber` that writes to stderr at `DEBUG`
//! level or above, before doing anything else, so `SKYJOUST_VALIDATOR_DEBUG=1`
//! (see `skyjoust_stateright_validator::transitions`) has somewhere to send
//! its `tracing::debug!` events; without a subscriber configured for at least
//! `DEBUG`, those events are silently dropped (a bare `fmt` subscriber only
//! shows `INFO` and above by default). Writing to stderr, not the default
//! stdout, keeps the pretty JSON result on stdout the sole machine-readable
//! output.

use std::{
    env,
    io::{self, Read},
};

use eyre::{Context, Report, bail};
use skyjoust_stateright_validator::{SkyAction, SkyjoustInteractionModel, validate_trace};

fn main() -> Result<(), Report> {
    tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let options = TraceCliOptions::parse(env::args().skip(1))?;
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .wrap_err("failed to read JSON trace from stdin")?;

    let trace: Vec<SkyAction> =
        serde_json::from_str(&input).wrap_err("failed to parse JSON trace as SkyAction list")?;
    let model = options.model();

    if options.verbose {
        trace_steps(&trace);
    }

    let result = validate_trace(&model, trace);

    if options.verbose {
        trace_final_state(&result);
    }

    let output = serde_json::to_string_pretty(&result)
        .wrap_err("failed to serialize trace validation result")?;
    print_result(&output);
    if result.ok {
        Ok(())
    } else {
        std::process::exit(2)
    }
}

/// Write one `--verbose` diagnostic line per replayed action to stderr.
#[expect(
    clippy::print_stderr,
    reason = "the --verbose flag's whole purpose is to print replay diagnostics to stderr"
)]
fn trace_steps(trace: &[SkyAction]) {
    for (step_index, action) in trace.iter().enumerate() {
        eprintln!("trace step {step_index}: {action:?}");
    }
}

/// Write a `--verbose` summary of the final replayed state to stderr.
#[expect(
    clippy::print_stderr,
    reason = "the --verbose flag's whole purpose is to print replay diagnostics to stderr"
)]
fn trace_final_state(result: &skyjoust_stateright_validator::TraceValidation) {
    eprintln!(
        "trace final state: ok={} depth={} app={:?} match_phase={:?} rewards={:?}",
        result.ok,
        result.final_state.depth,
        result.final_state.app,
        result.final_state.match_phase,
        result.final_state.rewards.phase
    );
}

/// Write the machine-readable `TraceValidation` JSON result to stdout.
#[expect(
    clippy::print_stdout,
    reason = "the machine-readable TraceValidation result is this CLI's stdout contract"
)]
fn print_result(output: &str) {
    println!("{output}");
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct TraceCliOptions {
    max_depth: Option<u16>,
    verbose: bool,
}

impl TraceCliOptions {
    fn parse<I, S>(args: I) -> Result<Self, Report>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut arg_values = args.into_iter().map(Into::into);
        let mut options = Self::default();

        while let Some(arg) = arg_values.next() {
            match arg.as_str() {
                "--max-depth" => {
                    let raw_depth = arg_values
                        .next()
                        .ok_or_else(|| eyre::eyre!("--max-depth requires a numeric value"))?;
                    options.max_depth = Some(
                        raw_depth
                            .parse::<u16>()
                            .wrap_err_with(|| format!("invalid --max-depth value: {raw_depth}"))?,
                    );
                }
                "--verbose" => {
                    options.verbose = true;
                }
                _ => bail!("unrecognised argument: {arg}"),
            }
        }

        Ok(options)
    }

    fn model(self) -> SkyjoustInteractionModel {
        self.max_depth
            .map_or_else(SkyjoustInteractionModel::default, |max_depth| {
                SkyjoustInteractionModel { max_depth }
            })
    }
}

#[cfg(test)]
mod tests {
    //! Tests for command-line option parsing.

    use super::*;

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "assert_eq! gives a clearer failure message than manually building an Err here"
    )]
    fn max_depth_overrides_default_model() -> Result<(), Report> {
        let options = TraceCliOptions::parse(["--max-depth", "40"])?;

        assert_eq!(options.model().max_depth, 40);
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "assert_eq! gives a clearer failure message than manually building an Err here"
    )]
    fn omitted_max_depth_uses_default_model() -> Result<(), Report> {
        let options = TraceCliOptions::parse(std::iter::empty::<&str>())?;

        assert_eq!(
            options.model().max_depth,
            SkyjoustInteractionModel::default().max_depth
        );
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "assert! gives a clearer failure message than manually building an Err here"
    )]
    fn verbose_flag_is_recorded() -> Result<(), Report> {
        let options = TraceCliOptions::parse(["--verbose"])?;

        assert!(options.verbose);
        Ok(())
    }

    #[test]
    fn missing_max_depth_fails() {
        let result = TraceCliOptions::parse(["--max-depth"]);

        assert!(result.is_err());
    }

    #[test]
    fn invalid_max_depth_fails() {
        let result = TraceCliOptions::parse(["--max-depth", "foo"]);

        assert!(result.is_err());
    }

    #[test]
    fn unrecognized_argument_fails() {
        let result = TraceCliOptions::parse(["--unknown"]);

        assert!(result.is_err());
    }
}
