use anchor_lang::prelude::*;

#[program]
pub mod references {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>) -> ProgramResult {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'a, 'info> {
    my_account: &'a AccountInfo<'info>,
}
