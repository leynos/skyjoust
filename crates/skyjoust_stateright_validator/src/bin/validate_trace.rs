use std::io::{self, Read};

use skyjoust_stateright_validator::{validate_trace, SkyAction, SkyjoustInteractionModel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let trace: Vec<SkyAction> = serde_json::from_str(&input)?;
    let model = SkyjoustInteractionModel::default();
    let result = validate_trace(&model, trace);

    println!("{}", serde_json::to_string_pretty(&result)?);
    if result.ok {
        Ok(())
    } else {
        std::process::exit(2)
    }
}
