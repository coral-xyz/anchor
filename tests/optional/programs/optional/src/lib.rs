//! This example demonstrates the ability to use optional accounts in
//! structs deriving `Accounts`.

use anchor_lang::prelude::*;
use processor::*;

pub mod errors;
pub use errors::OptionalErrors;
pub mod processor;
pub mod state;
declare_id!("FNqz6pqLAwvMSds2FYjR4nKV3moVpPNtvkfGFrqLKrgG");

#[program]
mod optional {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, value: u64, key: Pubkey) -> Result<()> {
        handle_initialize(ctx, value, key)
    }

    pub fn update(ctx: Context<Update>, value: u64, key: Pubkey) -> Result<()> {
        handle_update(ctx, value, key)
    }

    pub fn realloc(ctx: Context<Realloc>) -> Result<()> {
        handle_realloc(ctx)
    }

    pub fn close(ctx: Context<Close>) -> Result<()> {
        handle_close(ctx)
    }
}
