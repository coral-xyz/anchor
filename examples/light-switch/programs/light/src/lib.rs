// Light and Light Switch program with one to one relationship.
// They share the same switch account and only transaction signed by that account is able to turn light on and off.

// This is a community example and might not represent the best practices.

use anchor_lang::prelude::*;

#[program]
pub mod light {
    use super::*;

    #[state]
    pub struct Light {
        is_light_on: bool,
        switch: Pubkey
    }

    impl Light {
        pub fn new(ctx: Context<Init>) -> Result<Self> {
            Ok(Self {
                is_light_on: false,
                switch: *ctx.accounts.switch.key
            })
        }

        pub fn flip(&mut self, ctx: Context<FlipSwitch>) -> Result<()> {
            // Check the switch passed in is also the switch stored.
            if &self.switch != ctx.accounts.switch.key {
                return Err(ErrorCode::Unauthorized.into());
            }
            self.is_light_on = !self.is_light_on;
            Ok(())
        }
    }
}

#[derive(Accounts)]
pub struct Init<'info>{
    switch:AccountInfo<'info>
}

#[derive(Accounts)]
pub struct FlipSwitch<'info> {
    // Verifies master is the signer.
    #[account(signer)]
    pub switch: AccountInfo<'info>,
}

#[error]
pub enum ErrorCode {
    #[msg("You are not authorized to perform this action.")]
    Unauthorized,
}
