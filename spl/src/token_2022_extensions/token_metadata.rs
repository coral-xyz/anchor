use anchor_lang::solana_program::account_info::AccountInfo;
use anchor_lang::solana_program::pubkey::Pubkey;
use anchor_lang::Result;
use anchor_lang::{context::CpiContext, Accounts};

use borsh::{BorshDeserialize, BorshSerialize};
use spl_pod::optional_keys::OptionalNonZeroPubkey;
use spl_token_metadata_interface::state::Field;

#[derive(Clone, BorshDeserialize, BorshSerialize)]
pub struct TokenMetadataInitializeArgs {
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

pub fn token_metadata_initialize<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, TokenMetadataInitialize<'info>>,
    args: TokenMetadataInitializeArgs,
) -> Result<()> {
    let ix = spl_token_metadata_interface::instruction::initialize(
        ctx.accounts.token_program_id.key,
        ctx.accounts.metadata.key,
        ctx.accounts.update_authority.key,
        ctx.accounts.mint.key,
        ctx.accounts.mint_authority.key,
        args.name,
        args.symbol,
        args.uri,
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

#[derive(Clone, BorshDeserialize, BorshSerialize)]
pub struct TokenMetadataUpdateFieldArgs {
    pub field: Field,
    pub value: String,
}

pub fn token_metadata_update_field<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, TokenMetadataUpdateField<'info>>,
    args: TokenMetadataUpdateFieldArgs,
) -> Result<()> {
    let ix = spl_token_metadata_interface::instruction::update_field(
        ctx.accounts.token_program_id.key,
        ctx.accounts.metadata.key,
        ctx.accounts.update_authority.key,
        args.field,
        args.value,
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
