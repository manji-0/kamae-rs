fn main() {
    let test_cases = trybuild::TestCases::new();
    test_cases.compile_fail("tests/compile_fail/*.rs");
}
