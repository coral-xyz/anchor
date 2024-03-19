use solana_program_test::ProgramTest;

#[test]
fn check_entrypoint() {
    let _pt = ProgramTest::new(
        "solana_program_test_compatibility",
        solana_program_test_compatibility::id(),
        None,
    );
}
