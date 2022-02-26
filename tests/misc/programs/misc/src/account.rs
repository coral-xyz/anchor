use anchor_lang::prelude::*;

macro_rules! size {
    ($name: ident, $size:expr) => {
        impl $name {
            pub const LEN: usize = $size;
        }
    };
}

pub const MAX_SIZE: usize = 10;

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
