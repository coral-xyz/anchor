use anchor_lang::prelude::*;

macro_rules! size {
    ($name: ident, $size:expr) => {
        impl $name {
            pub const LEN: usize = $size;
        }
    };
}

pub const MAX_SIZE: usize = 10;
pub const MAX_SIZE_U8: u8 = 11;

#[account]
pub struct Data {
    pub udata: u128, // 16
    pub idata: i128, // 16
}
size!(Data, 32);

#[account]
pub struct DataU16 {
    pub data: u16, // 2
}
size!(DataU16, 32);

#[account]
pub struct DataI8 {
    pub data: i8, // 1
}
size!(DataI8, 1);

#[account]
pub struct DataI16 {
    pub data: i16, // 2
}
size!(DataI16, 2);

#[account]
pub struct DataEnum {
    pub data: TestEnum, // 1 + 16
}
size!(DataEnum, 17);

#[account(zero_copy)]
pub struct DataZeroCopy {
    pub data: u16,    // 2
    pub _padding: u8, // 1
    pub bump: u8,     // 1
}
size!(DataZeroCopy, 4);

#[account]
pub struct DataWithFilter {
    pub authority: Pubkey,  // 32
    pub filterable: Pubkey, // 32
}
size!(DataWithFilter, 64);

#[account]
pub struct DataMultidimensionalArray {
    pub data: [[u8; 10]; 10], // 100
}
size!(DataMultidimensionalArray, 100);

#[account]
pub struct DataConstArraySize {
    pub data: [u8; MAX_SIZE], // 10
}
size!(DataConstArraySize, MAX_SIZE);

#[account]
pub struct DataConstCastArraySize {
    pub data_one: [u8; MAX_SIZE as usize],
    pub data_two: [u8; MAX_SIZE_U8 as usize],
}

#[account]
pub struct DataMultidimensionalArrayConstSizes {
    pub data: [[u8; MAX_SIZE_U8 as usize]; MAX_SIZE],
}

#[account]
pub struct CoolEnumWrapperAccount {
    pub my_enum: CoolEnum,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub enum CoolEnum {
    Variant1,
    Variant2 {
        config: u8,
        user_1: Pubkey,
        some_slot: u64,
    },
    Variant3 {
        config: u8,
        user_1: Pubkey,
        user_2: Pubkey,
        some_slot: u64,
    },
}

#[derive(Debug, Clone, Copy, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub enum TestEnum {
    First,
    Second { x: u64, y: u64 },
    TupleTest(u8, u8, u16, u16),
    TupleStructTest(TestStruct),
}

#[derive(Debug, Clone, Copy, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub struct TestStruct {
    pub data1: u8,
    pub data2: u16,
    pub data3: u32,
    pub data4: u64,
}
