use anchor_lang::prelude::*;

declare_id!("DzVuV6qMC2oJJEwerpYrenPDhTQuHQRfMe4LdCmMZJYK");

#[program]
mod basic_0 {
    use super::*;
    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
