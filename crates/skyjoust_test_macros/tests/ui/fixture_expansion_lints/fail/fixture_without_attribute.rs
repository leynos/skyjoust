//! Missing suppression leaves the fixture expansion lint failure visible.

#![deny(unused_braces)]

use rstest::fixture;

#[fixture]
fn seed() -> u32 { 7 }

fn main() {}
