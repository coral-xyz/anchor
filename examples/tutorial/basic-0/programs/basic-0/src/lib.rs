use anchor_lang::prelude::*;

declare_id!("D6cG3NZZXDZxbVdgVvN8V7oWym47Rja5itQna2mMt3ha");

#[program]
mod basic_0 {
    use super::*;
    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
