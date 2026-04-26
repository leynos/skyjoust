//! Validate a serialized Skyjoust action trace from standard input.
//!
//! The `main` entrypoint reads a JSON array of `SkyAction` values from stdin,
//! calls `validate_trace` with `SkyjoustInteractionModel`, prints a pretty JSON
//! `TraceValidation` result, and exits with code 2 when the trace is invalid.

use std::{
    env,
    io::{self, Read},
};

use eyre::{bail, Context, Report};
use skyjoust_stateright_validator::{validate_trace, SkyAction, SkyjoustInteractionModel};

fn main() -> Result<(), Report> {
    let options = TraceCliOptions::parse(env::args().skip(1))?;
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .wrap_err("failed to read JSON trace from stdin")?;

    let trace: Vec<SkyAction> =
        serde_json::from_str(&input).wrap_err("failed to parse JSON trace as SkyAction list")?;
    let model = options.model();
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct TraceCliOptions {
    max_depth: Option<u8>,
}

impl TraceCliOptions {
    fn parse<I, S>(args: I) -> Result<Self, Report>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut args = args.into_iter().map(Into::into);
        let mut options = Self::default();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--max-depth" => {
                    let raw_depth = args
                        .next()
                        .ok_or_else(|| eyre::eyre!("--max-depth requires a numeric value"))?;
                    options.max_depth = Some(
                        raw_depth
                            .parse::<u8>()
                            .wrap_err_with(|| format!("invalid --max-depth value: {raw_depth}"))?,
                    );
                }
                _ => bail!("unrecognised argument: {arg}"),
            }
        }

        Ok(options)
    }

    fn model(self) -> SkyjoustInteractionModel {
        match self.max_depth {
            Some(max_depth) => SkyjoustInteractionModel { max_depth },
            None => SkyjoustInteractionModel::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    //! Tests for command-line option parsing.

    use super::*;

    #[test]
    fn max_depth_overrides_default_model() -> Result<(), Report> {
        let options = TraceCliOptions::parse(["--max-depth", "40"])?;

        assert_eq!(options.model().max_depth, 40);
        Ok(())
    }

    #[test]
    fn omitted_max_depth_uses_default_model() -> Result<(), Report> {
        let options = TraceCliOptions::parse(std::iter::empty::<&str>())?;

        assert_eq!(
            options.model().max_depth,
            SkyjoustInteractionModel::default().max_depth
        );
        Ok(())
    }
}
