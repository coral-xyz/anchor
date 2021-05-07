// Light and Light Switch program with one to one relationship.
// This is a community example and might not represent the best practices.

use anchor_lang::prelude::*;

#[program]
pub mod light {
    use super::*;

    #[state]
    pub struct Light {
        // State of the light.
        is_light_on: bool,
        // Store the PDA for verification
        switch_signer: Pubkey,
    }

    impl Light {
        pub fn new(ctx: Context<Init>) -> Result<Self> {
            Ok(Self {
                is_light_on: false,
                switch_signer: *ctx.accounts.switch_signer.key
            })
        }

        // Two conditions for flip:
        // 1. Must be signed by ctx.accounts.switch_signer.
        // 2. ctx.accounts.switch_signer must equals switch_signer stored in Light's State.
        pub fn flip(&mut self, ctx: Context<FlipSwitch>) -> Result<()> {
            // Verify switch signer PDA passed in equals switch_signer stored.
            if &self.switch_signer != ctx.accounts.switch_signer.key {
                return Err(ErrorCode::Unauthorized.into());
            }
            self.is_light_on = !self.is_light_on;
            Ok(())
        }
    }
}

#[derive(Accounts)]
pub struct Init<'info>{
    switch_signer:AccountInfo<'info>
}

#[derive(Accounts)]
pub struct FlipSwitch<'info> {
    // Verifies master is the signer.
    #[account(signer)]
    pub switch_signer: AccountInfo<'info>,
}

#[error]
pub enum ErrorCode {
    #[msg("You are not authorized to perform this action.")]
    Unauthorized,
}
