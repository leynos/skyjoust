//! Serve Stateright Explorer for the Skyjoust interaction model.
//!
//! Run this example, then open `http://localhost:3000/` to inspect reachable
//! states and counterexample paths for the bounded model.

use skyjoust_stateright_validator::SkyjoustInteractionModel;
use stateright::Model;

const EXPLORER_HOST: &str = "localhost:3000";
const EXPLORER_URL: &str = "http://localhost:3000/";

fn initialise_tracing() {
    // `with_max_level(DEBUG)` is required for `SKYJOUST_VALIDATOR_DEBUG=1`'s
    // `tracing::debug!` events (see `transitions::trace_transition_attempt`)
    // to reach this subscriber at all; a bare `fmt` subscriber only shows
    // `INFO` and above by default.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_max_level(tracing::Level::DEBUG)
        .init();
}

fn log_explorer_start() {
    tracing::info!(
        host = EXPLORER_HOST,
        "attempting to serve Stateright Explorer"
    );
}

fn log_explorer_ready() {
    tracing::info!(
        url = EXPLORER_URL,
        "Stateright Explorer is serving Project Skyjoust"
    );
}

fn main() {
    initialise_tracing();
    log_explorer_start();

    let _checker = SkyjoustInteractionModel { max_depth: 18 }
        .checker()
        .serve(EXPLORER_HOST);

    log_explorer_ready();
    std::thread::park();
}
