/*
Light and Light Switch program with one to one relationship.
They share the same switch account and only transaction signed by that account is able to turn light on and off.

This is a community example and might not represent the best practices.
*/

use anchor_lang::prelude::*;
use light::light::{Light};
use light::{FlipSwitch};

#[program]
pub mod light_switch {
    use super::*;

    #[state]
    pub struct LightSiwtch {
        authority: Pubkey,
        switch: Pubkey
    }


    impl LightSiwtch {
        pub fn new(ctx: Context<SwitchInit>) -> Result<Self> {
            Ok(Self {
                authority: *ctx.accounts.authority.key,
                switch: *ctx.accounts.switch.key
            })
        }

        pub fn flip(&mut self, ctx: Context<StateCpi>, nonce: u8) -> Result<()> {

            // check authority is the signer
            // check owner being passed in is same owner in state
            if &self.authority != ctx.accounts.authority.key || &self.switch != ctx.accounts.switch.key{
                return Err(ErrorCode::Unauthorized.into());
            }

            // obtain the light program
            let cpi_program = ctx.accounts.light_program.clone();

            // set the payload, pass in switch
            let cpi_accounts = FlipSwitch {
                switch: ctx.accounts.switch.to_account_info()
            };

            // sign with master and nonce of PDA (owner signer)
            let seeds = &[
                ctx.accounts.switch.to_account_info().key.as_ref(),
                &[nonce],
            ];
            let signer = &[&seeds[..]];
            let ctx = ctx.accounts.cpi_state.context(cpi_program, cpi_accounts);

            // call cpi function with signer
            light::cpi::state::flip(ctx.with_signer(signer));
            Ok(())
        }
    }

}

#[derive(Accounts)]
pub struct SwitchInit<'info> {
    #[account(signer)]
    authority: AccountInfo<'info>,
    switch: AccountInfo<'info>
}

#[derive(Accounts)]
pub struct StateCpi<'info> {
    #[account(signer)]
    authority: AccountInfo<'info>,
    switch: AccountInfo<'info>,
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
