// #region code
use anchor_lang::prelude::*;

declare_id!("671JxDuZAWcUMMiGbq5YUDUXGBcyAB3tuDDQu7RHUA6h");

#[program]
pub mod basic_5 {
    use super::*;

    #[state]
    pub struct Counter {
        pub authority: Pubkey,
        pub count: u64,
    }

    impl Counter {
        pub fn new(ctx: Context<Auth>) -> anchor_lang::Result<Self> {
            Ok(Self {
                authority: *ctx.accounts.authority.key,
                count: 0,
            })
        }

        pub fn increment(&mut self, ctx: Context<Auth>) -> anchor_lang::Result<()> {
            if &self.authority != ctx.accounts.authority.key {
                return Err(error!(ErrorCode::Unauthorized));
            }
            self.count += 1;
            Ok(())
        }

        /// On this error
        pub fn increment_out_of_bounds(&mut self, ctx: Context<Auth>) -> anchor_lang::Result<()> {
            println!("{}", crate::id());
            if &self.authority != ctx.accounts.authority.key {
                return Err(error!(ErrorCode::Unauthorized));
            }
            self.count += 1;
            Ok(())
        }
    }
}

#[derive(Accounts)]
pub struct Auth<'info> {
    authority: Signer<'info>,
}

#[derive(Accounts)]
#[bot_tax(
    error_codes = [
        ErrorCode::Unauthorized
    ],
    base_fee = 1,
    pay_to = bot_tax
)]
pub struct BotTaxAuth<'info> {
    authority: Signer<'info>,
    /// CHECK: This account is not read from
    bot_tax: UncheckedAccount<'info>,
    system_program: Program<'info, System>,
}
// #endregion code

#[error_code]
pub enum ErrorCode {
    #[msg("You are not authorized to perform this action.")]
    Unauthorized,
}
