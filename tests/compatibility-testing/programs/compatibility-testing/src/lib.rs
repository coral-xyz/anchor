use anchor_lang::prelude::*;

declare_id!("DhFkMBzU3BqvcWuUZkFQR8QajQsiGYLPrhCh5BQBgmkg");

#[program]
pub mod compatibility_testing {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
