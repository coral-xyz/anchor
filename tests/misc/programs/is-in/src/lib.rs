use anchor_lang::prelude::*;
use anchor_lang::ZeroCopy;
use bytemuck::{Pod, Zeroable};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod is_in {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        mint: Pubkey,
        vault: Pubkey,
        config: Pubkey,
    ) -> anchor_lang::Result<()> {
        *ctx.accounts.market = Market {
            authority: ctx.accounts.authority.key(),
            mint,
            vault,
            config,
        };

        *ctx.accounts.zero_market.load_init()? = Market {
            authority: ctx.accounts.authority.key(),
            mint,
            vault,
            config,
        };

        Ok(())
    }

    pub fn update(
        ctx: Context<Update>,
        mint: Pubkey,
        vault: Pubkey,
        config: Pubkey,
    ) -> anchor_lang::Result<()> {
        *ctx.accounts.market = Market {
            authority: ctx.accounts.authority.key(),
            mint,
            vault,
            config,
        };

        *ctx.accounts.zero_market.load_mut()? = Market {
            authority: ctx.accounts.authority.key(),
            mint,
            vault,
            config,
        };

        Ok(())
    }
    pub fn close(_ctx: Context<Close>) -> anchor_lang::Result<()> {
        Ok(())
    }
}

#[account]
#[derive(Copy)]
pub struct Market {
    authority: Pubkey,
    mint: Pubkey,
    vault: Pubkey,
    config: Pubkey,
}

unsafe impl Zeroable for Market {}
unsafe impl Pod for Market {}
impl ZeroCopy for Market {}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init,
        payer = authority,
        space = 8 + std::mem::size_of::<Market>(),
        seeds = [b"MARKET"],
        bump,
    )]
    pub market: Account<'info, Market>,

    #[account(init,
        payer = authority,
        space = 8 + std::mem::size_of::<Market>(),
        seeds = [b"ZERO_MARKET"],
        bump,
    )]
    pub zero_market: AccountLoader<'info, Market>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Update<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub zero_market: AccountLoader<'info, Market>,

    #[account(is_in = market, is_in = zero_market)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut, close = authority)]
    pub market: Box<Account<'info, Market>>,
    #[account(mut, close = authority)]
    pub zero_market: AccountLoader<'info, Market>,

    #[account(mut, is_in = market, is_in = zero_market)]
    pub authority: Signer<'info>,
    #[account(is_in = market, is_in = zero_market)]
    pub mint: UncheckedAccount<'info>,
    #[account(is_in = market, is_in = zero_market)]
    pub vault: UncheckedAccount<'info>,
    #[account(is_in = market, is_in = zero_market)]
    pub config: UncheckedAccount<'info>,
}
