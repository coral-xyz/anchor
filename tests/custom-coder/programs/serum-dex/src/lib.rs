use anchor_lang::prelude::*;
use borsh::{BorshDeserialize, BorshSerialize};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// This program is simply used to generate the IDL for the token program.
//
// Note that we manually add the COption<Pubkey> type to the IDL after
// compiling.
//
#[program]
pub mod serum_dex {
    use super::*;

    pub fn create_market(
        ctx: Context<CreateMarket>,
        signer_nonce: u64,
        min_base_order_size: u64,
        tick_size: u64,
        cranker_reward: u64,
    ) -> Result<()> {
        Ok(())
    }

    pub fn new_order(
        ctx: Context<NewOrder>,
        client_order_id: u128,
        limit_price: u64,
        max_base_qty: u64,
        max_quote_qty: u64,
        match_limit: u64,
        side: u8,
        order_type: u8,
        self_trade_behaviour: u8,
        has_discount_token_account: u8,
    ) -> Result<()> {
        Ok(())
    }

    pub fn swap(
        ctx: Context<Swap>,
        base_qty: u64,
        quote_qty: u64,
        match_limit: u64,
        side: u8,
        has_discount_token_account: u8,
        _padding: [u8; 6],
    ) -> Result<()> {
        Ok(())
    }

    pub fn cancel_order(ctx: Context<CancelOrder>, order_index: u64, order_id: u128) -> Result<()> {
        Ok(())
    }

    pub fn consume_events(
        ctx: Context<ConsumeEvents>,
        max_iterations: u64,
        no_op_err: u64,
    ) -> Result<()> {
        Ok(())
    }

    pub fn settle(ctx: Context<Settle>) -> Result<()> {
        Ok(())
    }

    pub fn initialize_account(
        ctx: Context<InitializeAccount>,
        market: Pubkey,
        max_orders: u64,
    ) -> Result<()> {
        Ok(())
    }

    pub fn sweep_fees(ctx: Context<SweepFees>) -> Result<()> {
        Ok(())
    }

    pub fn close_account(ctx: Context<CloseAccount>) -> Result<()> {
        Ok(())
    }

    pub fn close_market(ctx: Context<CloseMarket>) -> Result<()> {
        Ok(())
    }

    pub fn update_royalties(ctx: Context<UpdateRoyalties>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateMarket<'info> {
    #[account(mut)]
    market: AccountInfo<'info>,
    #[account(mut)]
    orderbook: AccountInfo<'info>,
    base_vault: AccountInfo<'info>,
    quote_vault: AccountInfo<'info>,
    market_admin: AccountInfo<'info>,
    #[account(mut)]
    event_queue: AccountInfo<'info>,
    #[account(mut)]
    asks: AccountInfo<'info>,
    #[account(mut)]
    bids: AccountInfo<'info>,
    token_metadata: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct NewOrder<'info> {
    spl_token_program: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    #[account(mut)]
    market: AccountInfo<'info>,
    #[account(mut)]
    orderbook: AccountInfo<'info>,
    #[account(mut)]
    event_queue: AccountInfo<'info>,
    #[account(mut)]
    bids: AccountInfo<'info>,
    #[account(mut)]
    asks: AccountInfo<'info>,
    #[account(mut)]
    base_vault: AccountInfo<'info>,
    #[account(mut)]
    quote_vault: AccountInfo<'info>,
    #[account(mut)]
    user: AccountInfo<'info>,
    #[account(mut)]
    user_token_account: AccountInfo<'info>,
    #[account(mut)]
    user_owner: Signer<'info>,
    // Remaining Accounts: discount_token_account, fee_referral_account
}

#[derive(Accounts)]
pub struct Swap<'info> {
    spl_token_program: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    #[account(mut)]
    market: AccountInfo<'info>,
    #[account(mut)]
    orderbook: AccountInfo<'info>,
    #[account(mut)]
    event_queue: AccountInfo<'info>,
    #[account(mut)]
    bids: AccountInfo<'info>,
    #[account(mut)]
    asks: AccountInfo<'info>,
    #[account(mut)]
    base_vault: AccountInfo<'info>,
    #[account(mut)]
    quote_vault: AccountInfo<'info>,
    market_signer: AccountInfo<'info>,
    #[account(mut)]
    user_base_account: AccountInfo<'info>,
    #[account(mut)]
    user_quote_account: AccountInfo<'info>,
    #[account(mut)]
    user_owner: Signer<'info>,
    // Remaining Accounts: discount_token_account, fee_referral_account
}

#[derive(Accounts)]
pub struct CancelOrder<'info> {
    market: AccountInfo<'info>,
    #[account(mut)]
    orderbook: AccountInfo<'info>,
    #[account(mut)]
    event_queue: AccountInfo<'info>,
    #[account(mut)]
    bids: AccountInfo<'info>,
    #[account(mut)]
    asks: AccountInfo<'info>,
    #[account(mut)]
    user: AccountInfo<'info>,
    user_owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct ConsumeEvents<'info> {
    #[account(mut)]
    market: AccountInfo<'info>,
    #[account(mut)]
    orderbook: AccountInfo<'info>,
    #[account(mut)]
    event_queue: AccountInfo<'info>,
    #[account(mut)]
    reward_target: AccountInfo<'info>,
    // Remaining Accounts: [...user_accounts]
}

#[derive(Accounts)]
pub struct Settle<'info> {
    spl_token_program: AccountInfo<'info>,
    market: AccountInfo<'info>,
    #[account(mut)]
    base_vault: AccountInfo<'info>,
    #[account(mut)]
    quote_vault: AccountInfo<'info>,
    market_signer: AccountInfo<'info>,
    #[account(mut)]
    user: AccountInfo<'info>,
    user_owner: Signer<'info>,
    #[account(mut)]
    destination_base_account: AccountInfo<'info>,
    #[account(mut)]
    destination_quote_account: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct InitializeAccount<'info> {
    system_program: AccountInfo<'info>,
    #[account(mut)]
    user: AccountInfo<'info>,
    user_owner: Signer<'info>,
    #[account(mut)]
    fee_payer: Signer<'info>,
}

#[derive(Accounts)]
pub struct SweepFees<'info> {
    #[account(mut)]
    market: AccountInfo<'info>,
    market_signer: AccountInfo<'info>,
    // sweep_authority: Signer<'info>,
    #[account(mut)]
    quote_vault: AccountInfo<'info>,
    #[account(mut)]
    destination_token_account: AccountInfo<'info>,
    spl_token_program: AccountInfo<'info>,
    token_metadata: AccountInfo<'info>,
    // Remaining Accounts: [...creators_token_accounts]
}

#[derive(Accounts)]
pub struct CloseAccount<'info> {
    #[account(mut)]
    user: AccountInfo<'info>,
    user_owner: Signer<'info>,
    #[account(mut)]
    target_lamports_account: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct CloseMarket<'info> {
    #[account(mut)]
    market: AccountInfo<'info>,
    #[account(mut)]
    base_vault: AccountInfo<'info>,
    #[account(mut)]
    quote_vault: AccountInfo<'info>,
    #[account(mut)]
    orderbook: AccountInfo<'info>,
    #[account(mut)]
    event_queue: AccountInfo<'info>,
    #[account(mut)]
    bids: AccountInfo<'info>,
    #[account(mut)]
    asks: AccountInfo<'info>,
    market_admin: Signer<'info>,
    #[account(mut)]
    target_lamports_account: AccountInfo<'info>,
    market_signer: AccountInfo<'info>,
    spl_token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct UpdateRoyalties<'info> {
    #[account(mut)]
    market: AccountInfo<'info>,
    event_queue: AccountInfo<'info>,
    orderbook: AccountInfo<'info>,
    token_metadata: AccountInfo<'info>,
}

#[account]
pub struct MarketState {
    pub tag: u64,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub orderbook: Pubkey,
    pub admin: Pubkey,
    pub creation_timestamp: i64,
    pub base_volume: u64,
    pub quote_volume: u64,
    pub accumulated_fees: u64,
    pub min_base_order_size: u64,
    pub royalties_bps: u64,
    pub accumulated_royalties: u64,
    pub signer_nonce: u8,
    pub fee_type: u8,
    pub _padding: [u8; 6],
}
