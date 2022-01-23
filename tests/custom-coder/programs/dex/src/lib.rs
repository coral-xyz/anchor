use anchor_lang::prelude::*;
use serum_dex::matching::{OrderType as DexOrderType, Side as DexSide};
use anchor_lang::solana_program::account_info::AccountInfo;
use anchor_lang::solana_program::entrypoint::ProgramResult;
use crate::serum_dex::instruction::{SelfTradeBehavior as DexSelfTradeBehavior};
pub use serum_dex;
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod dex {
    use super::*;
    #[allow(clippy::too_many_arguments)]
    pub fn initialize_market<'info>(ctx: Context<InitializeMarket>, 
        coin_lot_size: u64, 
        pc_lot_size: u64, 
        vault_signer_nonce: u64, 
        pc_dust_threshold: u64, 
        fee_rate_bps: u16,
        prune_authority: Pubkey,
        consume_events_authority: Pubkey,
        authority: Pubkey, ) -> ProgramResult {
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    pub fn new_order<'info>(ctx: Context<NewOrder>, 
        side: Side, 
        limit_price: u64, 
        max_coin_qty: u64,
        order_type: OrderType,
        client_order_id: u64,
        self_trade_behavior: SelfTradeBehavior,
        open_orders_authority: Pubkey,
        limit: u16,
        max_native_pc_qty_including_fees: u64) -> ProgramResult{
            Ok(())
    }
    pub fn new_order_v3<'info>(ctx: Context<NewOrderv3>, 
        side: Side, 
        limit_price: u64, 
        max_coin_qty: u64,
        self_trade_behaviour: SelfTradeBehavior,  
        order_type: OrderType,
        client_order_id: u64,
        open_orders_authority: Pubkey, 
        limit: u16) -> ProgramResult {
        Ok(())
    }
    pub fn match_orders<'info>(ctx: Context<MatchOrders>, limit: u16) -> ProgramResult {
        Ok(())
    }
    pub fn consume_events<'info>(ctx: Context<ConsumeEvents>, limit: u16) -> ProgramResult {
        Ok(())
    }
    pub fn cancel_order<'info>(ctx: Context<CancelOrder>, side: Side, order_id: u128, open_orders_authority: Pubkey) -> ProgramResult {
        Ok(())
    }
    pub fn settle_funds<'info>(ctx: Context<SettleFunds>, open_orders_authority: Pubkey) -> ProgramResult {
        Ok(())
    }
    pub fn disable_market(ctx: Context<DisableMarket>, disable_authority_key: Pubkey) -> ProgramResult {
        Ok(())
    }
    pub fn sweep_fees<'info>(ctx: Context<SweepFees>, sweep_authority: Pubkey) -> ProgramResult {
        Ok(())
    }
   
  
    pub fn cancel_order_v2(ctx: Context<CancelOrderv2>, side: Side, order_id: u128, open_orders_authority: Pubkey) -> ProgramResult {
        Ok(())
    }
    pub fn cancel_order_by_client_v2(ctx: Context<CancelOrderByClientv2>, client_id: u64, open_orders_authority: Pubkey) -> ProgramResult {
        Ok(())
    }
    pub fn send_take(ctx: Context<SendTake>, 
        side: Side, 
        limit_price: u64, 
        max_coin_qty: u64, 
        max_native_pc_qty_including_fees: u64,
        min_coin_qty: u64,
        min_native_pc_qty: u64,
        limit: u16) -> ProgramResult {
        Ok(())
    }
    pub fn close_open_orders(ctx: Context<CloseOpenOrders>, open_orders_authority: Pubkey) -> ProgramResult {
        Ok(())
    }
    pub fn init_open_orders(ctx: Context<InitOpenOrders>, open_orders_authority: Pubkey, market_authority: Pubkey) -> ProgramResult {
        Ok(())
    }
    pub fn prune(ctx: Context<Prune>, limit: u16, prune_authority: Pubkey) -> ProgramResult {
        Ok(())
    }
    pub fn consume_events_permissioned(ctx: Context<ConsumeEventsPermissioned>, limit: u16, consume_events_authority: Pubkey) -> ProgramResult {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeMarket<'info>{
    #[account(mut)]
    pub market: AccountInfo<'info>,
    pub coin_mint: AccountInfo<'info>,
    pub pc_mint: AccountInfo<'info>,
    #[account(mut)]
    pub coin_vault: AccountInfo<'info>,
    #[account(mut)]
    pub pc_vault: AccountInfo<'info>,
    #[account(mut)]
    pub request_queue: AccountInfo<'info>,
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,
    #[account(mut)]
    pub market_bids: AccountInfo<'info>,
    #[account(mut)]
    pub market_asks: AccountInfo<'info>,
    pub rent: AccountInfo<'info>,
}
#[derive(Accounts)]
pub struct NewOrder<'info>{
    #[account(mut)]
    pub market: AccountInfo<'info>,
    #[account(mut)]
    pub open_orders: AccountInfo<'info>,
    #[account(mut)]
    pub request_queue: AccountInfo<'info>,
    pub event_queue: AccountInfo<'info>,
    pub market_bids: AccountInfo<'info>,
    pub market_asks: AccountInfo<'info>,
    #[account(mut)]
    pub order_payer_token_account: AccountInfo<'info>,
    pub coin_vault: AccountInfo<'info>,
    pub pc_vault: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub rent: AccountInfo<'info>,
    pub srm_account_referral: AccountInfo<'info>,

}
#[derive(Accounts)]
pub struct NewOrderv3<'info>{
    #[account(mut)]
    pub market: AccountInfo<'info>,
    #[account(mut)]
    pub open_orders: AccountInfo<'info>,
    #[account(mut)]
    pub request_queue: AccountInfo<'info>,
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,
    #[account(mut)]
    pub market_bids: AccountInfo<'info>,
    #[account(mut)]
    pub market_asks: AccountInfo<'info>,
    #[account(mut)]
    pub order_payer_token_account: AccountInfo<'info>,
    #[account(mut)]
    pub coin_vault: AccountInfo<'info>,
    #[account(mut)]
    pub pc_vault: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub rent: AccountInfo<'info>,


}
#[derive(Accounts)]
pub struct MatchOrders<'info> {
    #[account(mut)]
    pub market: AccountInfo<'info>,
    #[account(mut)]
    pub req_queue: AccountInfo<'info>,
    #[account(mut)]
    pub market_bids: AccountInfo<'info>,
    #[account(mut)]
    pub market_asks: AccountInfo<'info>,
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,
    pub coin_fee_receivable_account: AccountInfo<'info>,
    pub pc_fee_receivable_account: AccountInfo<'info>,
}
#[derive(Accounts)]
pub struct ConsumeEvents<'info> {
    #[account(mut)]
    pub open_orders: AccountInfo<'info>,
    #[account(mut)]
    pub market: AccountInfo<'info>,
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,
    pub coin_fee_receivable_account: AccountInfo<'info>,
    pub pc_fee_receivable_account: AccountInfo<'info>,
}
    

#[derive(Accounts)]
pub struct CancelOrder<'info>{
    pub market: AccountInfo<'info>,
    pub market_bids: AccountInfo<'info>,
    pub market_asks: AccountInfo<'info>,
    #[account(mut)]
    pub open_orders: AccountInfo<'info>,
    #[account(mut)]
    pub req_queue: AccountInfo<'info>,
    pub event_queue: AccountInfo<'info>,

}
#[derive(Accounts)]
pub struct SettleFunds<'info>{
    #[account(mut)]
    pub market: AccountInfo<'info>,
    #[account(mut)]
    pub open_orders: AccountInfo<'info>,
    #[account(mut)]
    pub coin_wallet: AccountInfo<'info>,
    #[account(mut)]
    pub pc_wallet: AccountInfo<'info>,
    #[account(mut)]
    pub coin_vault: AccountInfo<'info>,
    #[account(mut)]
    pub pc_vault: AccountInfo<'info>,
    pub vault_signer: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct DisableMarket<'info>{
    #[account(mut)]
    pub market: AccountInfo<'info>,
    
}
#[derive(Accounts)]
pub struct SweepFees<'info>{ 
    #[account(mut)]
    pub market: AccountInfo<'info>,
    #[account(mut)]
    pub pc_vault: AccountInfo<'info>,
    #[account(mut)]
    pub sweep_receiver_amount: AccountInfo<'info>,
    pub vault_signer: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct CancelOrderv2<'info>{
    #[account(mut)]
    pub market: AccountInfo<'info>,
    #[account(mut)]
    pub market_bids: AccountInfo<'info>,
    #[account(mut)]
    pub market_asks: AccountInfo<'info>,
    #[account(mut)]
    pub open_orders: AccountInfo<'info>,
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,

}
#[derive(Accounts)]
pub struct CancelOrderByClientv2<'info>{
    #[account(mut)]
    pub market: AccountInfo<'info>,
    #[account(mut)]
    pub market_bids: AccountInfo<'info>,
    #[account(mut)]
    pub market_asks: AccountInfo<'info>,
    #[account(mut)]
    pub open_orders: AccountInfo<'info>,
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,

}
#[derive(Accounts)]
pub struct SendTake<'info>{
    #[account(mut)]
    pub market: AccountInfo<'info>,
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,
    #[account(mut)]
    pub market_bids: AccountInfo<'info>,
    #[account(mut)]
    pub market_asks: AccountInfo<'info>,
    #[account(mut)]
    pub coin_wallet: AccountInfo<'info>,
    #[account(mut)]
    pub pc_wallet: AccountInfo<'info>,
    #[account(mut)]
    pub req_queue: AccountInfo<'info>,

}
#[derive(Accounts)]
pub struct CloseOpenOrders<'info>{
    #[account(mut)]
    pub open_orders: AccountInfo<'info>,
    #[account(mut)]
    pub destination: AccountInfo<'info>,
    pub market: AccountInfo<'info>,

}
#[derive(Accounts)]
pub struct InitOpenOrders<'info>{
    #[account(mut)]
    pub open_orders: AccountInfo<'info>,
    pub market: AccountInfo<'info>,

}
#[derive(Accounts)]
pub struct Prune<'info>{
    #[account(mut)]
    pub market: AccountInfo<'info>,
    #[account(mut)]
    pub market_bids: AccountInfo<'info>,
    #[account(mut)]
    pub market_asks: AccountInfo<'info>,
    pub open_orders: AccountInfo<'info>,
    pub open_orders_authority: AccountInfo<'info>,
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,
}
#[derive(Accounts)]
pub struct ConsumeEventsPermissioned<'info>{
    pub open_orders: AccountInfo<'info>,
    #[account(mut)]
    pub market: AccountInfo<'info>,
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum Side {
    Bid,
    Ask,
}

impl From<Side> for DexSide{
    fn from(side:Side) -> DexSide{
        match side{
            Side::Bid => DexSide::Bid,
            Side::Ask => DexSide::Ask,
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum OrderType {
    Limit,
    ImmediateOrCancel,
    PostOnly,
}
impl From<OrderType> for DexOrderType{
    fn from(ordertype: OrderType) -> DexOrderType{
        match ordertype{
            OrderType:: Limit => DexOrderType::Limit,
            OrderType:: ImmediateOrCancel => DexOrderType::ImmediateOrCancel,
            OrderType:: PostOnly => DexOrderType::PostOnly,
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum SelfTradeBehavior{
    DecrementTake,
    CancelProvide,
    AbortTransaction,
}
impl From<SelfTradeBehavior> for DexSelfTradeBehavior{
    fn from(selftradebehavior: SelfTradeBehavior) ->DexSelfTradeBehavior{
        match selftradebehavior{
            SelfTradeBehavior:: DecrementTake => DexSelfTradeBehavior:: DecrementTake,
            SelfTradeBehavior:: CancelProvide => DexSelfTradeBehavior:: CancelProvide,
            SelfTradeBehavior:: AbortTransaction => DexSelfTradeBehavior:: AbortTransaction,
        }
    }
}
