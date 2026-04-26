//! Serve Stateright Explorer for the Skyjoust interaction model.
//!
//! Run this example, then open `http://localhost:3000/` to inspect reachable
//! states and counterexample paths for the bounded model.

use skyjoust_stateright_validator::SkyjoustInteractionModel;
use stateright::Model;

fn main() {
    // Then open http://localhost:3000/ to inspect reachable states and paths.
    let _checker = SkyjoustInteractionModel { max_depth: 18 }
        .checker()
        .serve("localhost:3000");

    println!("Stateright Explorer is serving Project Skyjoust at http://localhost:3000/");
    std::thread::park();
}
