use anchor_lang::prelude::*;

declare_id!("3cgdzWdfZSy1GaV6Lg98iwLvTcL9W7AVD8BpxiZjCZ9z");

#[program]
pub mod solana_program_test_compatibility {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
