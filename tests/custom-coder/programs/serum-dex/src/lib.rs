use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// This program is simply used to generate the IDL for the token program.
//
// Note that we manually add the COption<Pubkey> type to the IDL after
// compiling.
//
#[program]
pub mod serum_dex {
    use super::*;

    pub fn initialize_market(
        ctx: Context<InitializeMarket>,
        coin_lot_size: u64,
        pc_lot_size: u64,
        fee_rate_bps: u16,
        vault_signer_nonce: u64,
        pc_dust_threshold: u64,
    ) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeMarket<'info> {
    #[account(mut)]
    market: AccountInfo<'info>,
    #[account(mut)]
    request_queue: AccountInfo<'info>,
    #[account(mut)]
    event_queue: AccountInfo<'info>,
    #[account(mut)]
    bids: AccountInfo<'info>,
    #[account(mut)]
    asks: AccountInfo<'info>,
    #[account(mut)]
    coin_vault: AccountInfo<'info>,
    #[account(mut)]
    pc_vault: AccountInfo<'info>,
    coin_mint: AccountInfo<'info>,
    pc_mint: AccountInfo<'info>,
    rent: AccountInfo<'info>,
    market_authority: AccountInfo<'info>,
    prune_authority: AccountInfo<'info>,
    crank_authority: AccountInfo<'info>,
}
