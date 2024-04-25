use anchor_lang::{AnchorDeserialize, AnchorSerialize, Discriminator, InstructionData};

#[test]
fn test_instruction_data() {
    // Define some test type and implement ser/de, discriminator, and ix data
    #[derive(Default, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
    struct MyType {
        foo: [u8; 8],
        bar: String,
    }
    impl Discriminator for MyType {
        const DISCRIMINATOR: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
    }
    impl InstructionData for MyType {}

    // Initialize some instance of the type
    let instance = MyType {
        foo: [0, 2, 4, 6, 8, 10, 12, 14],
        bar: "sharding sucks".into(),
    };

    // Serialize using both methods
    let data = instance.data();
    let mut write = vec![];
    instance.write_to(&mut write);

    // Check that one is correct and that they are equal (implies other is correct)
    let correct_disc = data[0..8] == MyType::DISCRIMINATOR;
    let correct_data = MyType::deserialize(&mut &data[8..]).is_ok_and(|result| result == instance);
    let correct_serialization = correct_disc & correct_data;
    assert!(correct_serialization, "serialization was not correct");
    assert_eq!(
        &data, &write,
        "the different methods produced different serialized representations"
    );
}
