use anchor_lang::prelude::*;

#[account]
pub struct Data1 {
    pub data: u64,
}

impl Data1 {
    pub const LEN: usize = 8 + 8;
    pub const PREFIX: &'static str = "data1";
}

#[account]
pub struct Data2 {
    pub optional_1: Pubkey,
}

impl Data2 {
    pub const LEN: usize = 8 + 32;
}
