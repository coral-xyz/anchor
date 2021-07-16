// Note. This example depends on unreleased Serum DEX changes.

use anchor_lang::prelude::*;
use anchor_spl::dex::{
    Context, Logger, MarketMiddleware, MarketProxy, OpenOrdersPda, ReferralFees,
};
use serum_dex::instruction::{CancelOrderInstructionV2, NewOrderInstructionV3};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::rent;

/// A simplified version of the `programs/permissioned-markets` example, using
/// the `MarketProxy` abstraction.
///
/// To implement a custom proxy, one can implement the `MarketMiddleware` trait
/// to intercept, modify, and perform any access control on DEX requests before
/// they get forwarded to the orderbook.
///
/// These middleware can be mixed and matched. Note, however, that the order
/// of each middleware matters. Some like the `Identity` middleware provided
/// here expect different accounts and data.
///
#[program]
pub mod permissioned_markets_middleware {
    use super::*;
    pub fn entry(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
        MarketProxy::new()
            .middleware(&Logger)
            .middleware(&Identity)
            .middleware(&ReferralFees::new(referral::ID))
            .middleware(&OpenOrdersPda)
            .run(program_id, accounts, data)
    }
}

/// Performs token-based authorization, confirming the identity of the user.
/// The identity token must be given as the fist account.
struct Identity;

impl MarketMiddleware for Identity {
    /// Accounts:
    ///
    /// 0. Authorization token.
    /// ..
    fn init_open_orders(&self, ctx: &mut Context) -> ProgramResult {
        verify_and_strip_auth(ctx)
    }

    /// Accounts:
    ///
    /// 0. Authorization token.
    /// ..
    fn new_order_v3(&self, ctx: &mut Context, _ix: &NewOrderInstructionV3) -> ProgramResult {
        verify_and_strip_auth(ctx)
    }

    /// Accounts:
    ///
    /// 0. Authorization token.
    /// ..
    fn cancel_order_v2(&self, ctx: &mut Context, _ix: &CancelOrderInstructionV2) -> ProgramResult {
        verify_and_strip_auth(ctx)
    }

    /// Accounts:
    ///
    /// 0. Authorization token.
    /// ..
    fn cancel_order_by_client_id_v2(&self, ctx: &mut Context, _client_id: u64) -> ProgramResult {
        verify_and_strip_auth(ctx)
    }

    /// Accounts:
    ///
    /// 0. Authorization token.
    /// ..
    fn settle_funds(&self, ctx: &mut Context) -> ProgramResult {
        verify_and_strip_auth(ctx)
    }

    /// Accounts:
    ///
    /// 0. Authorization token.
    /// ..
    fn close_open_orders(&self, ctx: &mut Context) -> ProgramResult {
        verify_and_strip_auth(ctx)
    }

    /// Accounts:
    ///
    /// 0. Authorization token.
    /// ..
    fn fallback(&self, ctx: &mut Context) -> ProgramResult {
        verify_and_strip_auth(ctx)
    }
}

// Utils.

fn verify_and_strip_auth(ctx: &mut Context) -> ProgramResult {
    // The rent sysvar is used as a dummy example of an identity token.
    let auth = &ctx.accounts[0];
    require!(auth.key == &rent::ID, InvalidAuth);

    // Strip off the account before possing on the message.
    ctx.accounts = (&ctx.accounts[1..]).to_vec();

    Ok(())
}

// Error.

#[error]
pub enum ErrorCode {
    #[msg("Invalid auth token provided")]
    InvalidAuth,
}

// Consants.

pub mod referral {
    // This is a dummy address. Do not use in production.
    solana_program::declare_id!("2k1bb16Hu7ocviT2KC3wcCgETtnC8tEUuvFBH4C5xStG");
}
