use skyjoust_stateright_validator::SkyjoustInteractionModel;
use stateright::{Checker, Model};

#[test]
fn exhaustive_high_level_interaction_contract() {
    // Depth 18 covers: skirmish setup, tournament completion, duel completion,
    // truce break, keep breach, final score export, reward tally, and reward commit.
    let checker = SkyjoustInteractionModel { max_depth: 18 }
        .checker()
        .spawn_dfs()
        .join();

    checker.assert_properties();
}
