use solana_program_test::{processor, ProgramTest};

#[test]
fn entrypoint_lifetime() {
    let _pt = ProgramTest::new(
        "compatibility_testing",
        compatibility_testing::id(),
        processor!(compatibility_testing::entry),
    );
}
