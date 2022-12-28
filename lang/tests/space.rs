use anchor_lang::{prelude::*, Space};

// Needed to declare accounts.
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[derive(InitSpace)]
pub enum TestBasicEnum {
    Basic1,
    Basic2 {
        test_u8: u8,
    },
    Basic3 {
        test_u16: u16,
    },
    Basic4 {
        #[max_len(10)]
        test_vec: Vec<u8>,
    },
}

#[account]
pub struct TestEmptyAccount {}

#[account]
pub struct TestBasicVarAccount {
    pub test_u8: u8,
    pub test_u16: u16,
    pub test_u32: u32,
    pub test_u64: u64,
    pub test_u128: u128,
}

#[account]
pub struct TestComplexeVarAccount {
    pub test_key: Pubkey,
    #[max_len(10)]
    pub test_vec: Vec<u8>,
    #[max_len(10)]
    pub test_string: String,
}

#[derive(InitSpace)]
pub struct TestNonAccountStruct {
    pub test_bool: bool,
}

#[account(zero_copy)]
#[derive(InitSpace)]
pub struct TestZeroCopyStruct {
    pub test_array: [u8; 10],
    pub test_u32: u32,
}

#[derive(InitSpace)]
pub struct ChildStruct {
    #[max_len(10)]
    pub test_string: String,
}

#[derive(InitSpace)]
pub struct TestNestedStruct {
    pub test_struct: ChildStruct,
    pub test_enum: TestBasicEnum,
}

#[derive(InitSpace)]
pub struct TestMatrixStruct {
    #[max_len(2, 4)]
    pub test_matrix: Vec<Vec<u8>>,
}

#[test]
fn test_empty_struct() {
    assert_eq!(TestEmptyAccount::INIT_SPACE, 0);
}

#[test]
fn test_basic_struct() {
    assert_eq!(TestBasicVarAccount::INIT_SPACE, 1 + 2 + 4 + 8 + 16);
}

#[test]
fn test_complexe_struct() {
    assert_eq!(TestComplexeVarAccount::INIT_SPACE, 32 + 4 + 10 + (4 + 10))
}

#[test]
fn test_zero_copy_struct() {
    assert_eq!(TestZeroCopyStruct::INIT_SPACE, 10 + 4)
}

#[test]
fn test_basic_enum() {
    assert_eq!(TestBasicEnum::INIT_SPACE, 1 + 14);
}

#[test]
fn test_nested_struct() {
    assert_eq!(
        TestNestedStruct::INIT_SPACE,
        ChildStruct::INIT_SPACE + TestBasicEnum::INIT_SPACE
    )
}

#[test]
fn test_matrix_struct() {
    assert_eq!(TestMatrixStruct::INIT_SPACE, 4 + (2 * (4 + 4)))
}
