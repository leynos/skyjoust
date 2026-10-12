//! Compile-time coverage for the `rstest-bdd` attribute-macro contract.

#[test]
fn bdd_macro_contract_has_pass_and_failure_cases() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui/bdd_macro_contract.rs");
    cases.compile_fail("tests/ui/missing_feature_file.rs");
    cases.compile_fail("tests/ui/missing_include_file.rs");
}
