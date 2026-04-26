//! Validate a serialized Skyjoust action trace from standard input.
//!
//! The `main` entrypoint reads a JSON array of `SkyAction` values from stdin,
//! calls `validate_trace` with `SkyjoustInteractionModel`, prints a pretty JSON
//! `TraceValidation` result, and exits with code 2 when the trace is invalid.

use std::io::{self, Read};

use eyre::{Context, Report};
use skyjoust_stateright_validator::{validate_trace, SkyAction, SkyjoustInteractionModel};

fn main() -> Result<(), Report> {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .wrap_err("failed to read JSON trace from stdin")?;

    let trace: Vec<SkyAction> =
        serde_json::from_str(&input).wrap_err("failed to parse JSON trace as SkyAction list")?;
    let model = SkyjoustInteractionModel::default();
    let result = validate_trace(&model, trace);

    let output = serde_json::to_string_pretty(&result)
        .wrap_err("failed to serialize trace validation result")?;
    println!("{output}");
    if result.ok {
        Ok(())
    } else {
        std::process::exit(2)
    }
}
