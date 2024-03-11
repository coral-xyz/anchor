use anchor_lang::solana_program::account_info::AccountInfo;
use anchor_lang::solana_program::pubkey::Pubkey;
use anchor_lang::Result;
use anchor_lang::{context::CpiContext, Accounts};

use spl_pod::optional_keys::OptionalNonZeroPubkey;
use spl_token_metadata_interface::state::Field;

pub fn token_metadata_initialize<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, TokenMetadataInitialize<'info>>,
    name: String,
    symbol: String,
    uri: String,
) -> Result<()> {
    let ix = spl_token_metadata_interface::instruction::initialize(
        ctx.accounts.token_program_id.key,
        ctx.accounts.metadata.key,
        ctx.accounts.update_authority.key,
        ctx.accounts.mint.key,
        ctx.accounts.mint_authority.key,
        name,
        symbol,
        uri,
    );
    solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.token_program_id,
            ctx.accounts.metadata,
            ctx.accounts.update_authority,
            ctx.accounts.mint,
            ctx.accounts.mint_authority,
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Accounts)]
pub struct TokenMetadataInitialize<'info> {
    pub token_program_id: AccountInfo<'info>,
    pub metadata: AccountInfo<'info>,
    pub update_authority: AccountInfo<'info>,
    pub mint_authority: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
}

pub fn token_metadata_update_authority<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, TokenMetadataUpdateAuthority<'info>>,
    new_authority: OptionalNonZeroPubkey,
) -> Result<()> {
    let ix = spl_token_metadata_interface::instruction::update_authority(
        ctx.accounts.token_program_id.key,
        ctx.accounts.metadata.key,
        ctx.accounts.current_authority.key,
        new_authority,
    );
    solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.token_program_id,
            ctx.accounts.metadata,
            ctx.accounts.current_authority,
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Accounts)]
pub struct TokenMetadataUpdateAuthority<'info> {
    pub token_program_id: AccountInfo<'info>,
    pub metadata: AccountInfo<'info>,
    pub current_authority: AccountInfo<'info>,
    pub new_authority: AccountInfo<'info>,
}

pub fn token_metadata_update_field<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, TokenMetadataUpdateField<'info>>,
    field: Field,
    value: String,
) -> Result<()> {
    let ix = spl_token_metadata_interface::instruction::update_field(
        ctx.accounts.token_program_id.key,
        ctx.accounts.metadata.key,
        ctx.accounts.update_authority.key,
        field,
        value,
    );
    solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.token_program_id,
            ctx.accounts.metadata,
            ctx.accounts.update_authority,
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Accounts)]
pub struct TokenMetadataUpdateField<'info> {
    pub token_program_id: AccountInfo<'info>,
    pub metadata: AccountInfo<'info>,
    pub update_authority: AccountInfo<'info>,
}
