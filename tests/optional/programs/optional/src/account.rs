use anchor_lang::prelude::*;

#[account]
pub struct DataPda {
    pub data: u64,
}

impl DataPda {
    pub const LEN: usize = 8 + 8;
    pub const PREFIX: &'static str = "data_pda";
}

#[account]
pub struct DataAccount {
    pub data_pda: Pubkey,
}

impl DataAccount {
    pub const LEN: usize = 8 + 32;
}
