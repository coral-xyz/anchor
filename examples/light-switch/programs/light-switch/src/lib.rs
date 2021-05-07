// Light and Light Switch program with one to one relationship.
// They share the same switch account and only transaction signed by that account is able to turn light on and off.

// This is a community example and might not represent the best practices.

use anchor_lang::prelude::*;
use light::light::{Light};
use light::{FlipSwitch};

#[program]
pub mod light_switch {
    use super::*;

    #[state]
    pub struct LightSwitch {
        authority: Pubkey,
        nonce: u8
    }


    impl LightSwitch {
        pub fn new(ctx: Context<SwitchInit>, nonce: u8) -> Result<Self> {
            Ok(Self {
                authority: *ctx.accounts.authority.key,
                nonce
            })
        }

        pub fn flip(&mut self, ctx: Context<StateCpi>) -> Result<()> {

            // Check authority is the signer.
            if &self.authority != ctx.accounts.authority.key {
                return Err(ErrorCode::Unauthorized.into());
            }

            // Obtain the light program.
            let cpi_program = ctx.accounts.light_program.clone();

            // Set the instruction payload, pass in switch signer.
            let cpi_accounts = FlipSwitch {
                switch_signer: ctx.accounts.switch_signer.to_account_info()
            };
            let ctx = ctx.accounts.cpi_state.context(cpi_program, cpi_accounts);

            // Prepare the signature by switch + nonce to make the call sign by switch_signer.
            let seeds = &[
                ctx.accounts.switch.to_account_info().key.as_ref(),
                &[self.nonce],
            ];
            let signer = &[&seeds[..]];

            // Call cpi function with signer.
            light::cpi::state::flip(ctx.with_signer(signer));
            Ok(())
        }
    }

}

#[derive(Accounts)]
pub struct SwitchInit<'info> {
    #[account(signer)]
    authority: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct StateCpi<'info> {
    #[account(signer)]
    authority: AccountInfo<'info>,
    switch: AccountInfo<'info>,
    switch_signer: AccountInfo<'info>,
    #[account(mut, state = light_program)]
    cpi_state: CpiState<'info, Light>,
    #[account(executable)]
    light_program: AccountInfo<'info>,
}

#[error]
pub enum ErrorCode {
    #[msg("You are not authorized to perform this action.")]
    Unauthorized,
}
