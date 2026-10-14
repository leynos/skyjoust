//! Compile-fail fixture for a missing feature file in the rebuild dependency.

const _: &str = include_str!("features/missing.feature");

fn main() {}
