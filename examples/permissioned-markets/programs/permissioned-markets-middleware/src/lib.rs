// Note. This example depends on unreleased Serum DEX changes.

use anchor_lang::prelude::*;
use anchor_spl::dex::{
    Context, Logger, MarketMiddleware, MarketProxy, OpenOrdersPda, ReferralFees,
};
use serum_dex::instruction::{CancelOrderInstructionV2, NewOrderInstructionV3};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::pubkey::Pubkey;

/// A simplified version of the `programs/permissioned-markets` example, using
/// the `MarketProxy` abstraction.
///
/// To implement a custom proxy, one can implement the `MarketMiddleware` trait
/// to intercept, modify, and perform any access control on DEX requests before
/// they get forwarded to the orderbook.
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

// Performs token-based authorization, confirming the identity of the user.
// The identity token must be given as the fist account.
//
// TODO:
//   * Assume the identity token is given as the first account (on the client
//     this will technically be the second account, but that's stripped off
//     before hitting this code).
//   * Strip the tokens off the context.
//   * Validate the token.
struct Identity;
impl MarketMiddleware for Identity {
    fn init_open_orders(&self, _ctx: &mut Context) -> ProgramResult {
        Ok(())
    }

    fn new_order_v3(&self, _ctx: &mut Context, _ix: &NewOrderInstructionV3) -> ProgramResult {
        Ok(())
    }

    fn cancel_order_v2(&self, _ctx: &mut Context, _ix: &CancelOrderInstructionV2) -> ProgramResult {
        Ok(())
    }

    fn cancel_order_by_client_id_v2(&self, _ctx: &mut Context, _client_id: u64) -> ProgramResult {
        Ok(())
    }

    fn settle_funds(&self, _ctx: &mut Context) -> ProgramResult {
        Ok(())
    }

    fn close_open_orders(&self, _ctx: &mut Context) -> ProgramResult {
        Ok(())
    }

    fn fallback(&self, _ctx: &mut Context) -> ProgramResult {
        Ok(())
    }
}

pub mod referral {
    // This is a dummy address. Do not use in production.
    solana_program::declare_id!("2k1bb16Hu7ocviT2KC3wcCgETtnC8tEUuvFBH4C5xStG");
}
