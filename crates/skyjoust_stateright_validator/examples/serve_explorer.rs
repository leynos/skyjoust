//! Serve Stateright Explorer for the Skyjoust interaction model.
//!
//! Run this example, then open `http://localhost:3000/` to inspect reachable
//! states and counterexample paths for the bounded model.

use skyjoust_stateright_validator::SkyjoustInteractionModel;
use stateright::Model;

const EXPLORER_HOST: &str = "localhost:3000";
const EXPLORER_URL: &str = "http://localhost:3000/";

fn main() {
    eprintln!("attempting to serve Stateright Explorer at {EXPLORER_HOST}");
    let _checker = SkyjoustInteractionModel { max_depth: 18 }
        .checker()
        .serve(EXPLORER_HOST);

    println!("Stateright Explorer is serving Project Skyjoust at {EXPLORER_URL}");
    std::thread::park();
}
