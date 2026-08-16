//! Verifies the fixture lint macro's compile-time contract.

#[test]
fn fixture_expansion_lints_ui() {
    let test_cases = trybuild::TestCases::new();

    test_cases.pass("tests/ui/fixture_expansion_lints/pass/*.rs");
    test_cases.compile_fail("tests/ui/fixture_expansion_lints/fail/*.rs");
}
