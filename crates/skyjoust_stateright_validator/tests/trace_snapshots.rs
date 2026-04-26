//! Snapshot tests for trace validation output contracts.

use std::error::Error;

use skyjoust_stateright_validator::{SkyAction, SkyjoustInteractionModel, validate_trace};

#[test]
fn tournament_reward_commit_trace_matches_snapshot() -> Result<(), Box<dyn Error>> {
    let result = validate_fixture(include_str!("../traces/tournament_reward_commit.json"))?;

    insta::assert_json_snapshot!(result);
    Ok(())
}

#[test]
fn keep_breach_reward_commit_trace_matches_snapshot() -> Result<(), Box<dyn Error>> {
    let result = validate_fixture(include_str!("../traces/keep_breach_reward_commit.json"))?;

    insta::assert_json_snapshot!(result);
    Ok(())
}

#[test]
fn reward_commit_before_final_score_matches_failure_snapshot() {
    let model = SkyjoustInteractionModel::default();
    let result = validate_trace(
        &model,
        [
            SkyAction::AssetsLoaded,
            SkyAction::StartSkirmish,
            SkyAction::CommitRewards,
        ],
    );

    assert!(!result.ok);
    assert!(result.failure.is_some());
    insta::assert_json_snapshot!(result.failure);
}

fn validate_fixture(
    fixture: &str,
) -> Result<skyjoust_stateright_validator::TraceValidation, Box<dyn Error>> {
    let trace: Vec<SkyAction> = serde_json::from_str(fixture)?;

    Ok(validate_trace(&SkyjoustInteractionModel::default(), trace))
}
