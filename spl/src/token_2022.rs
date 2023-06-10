use anchor_lang::{
    context::CpiContext,
    prelude::UncheckedAccount,
    solana_program::{self, pubkey::Pubkey},
    Accounts, Result, ToAccountInfo,
};

pub use spl_token_2022;
pub use spl_token_2022::ID;

#[deprecated(
    since = "0.28.0",
    note = "please use `transfer_checked` or `transfer_checked_with_fee` instead"
)]
pub fn transfer<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, Transfer<'info>>,
    amount: u64,
) -> Result<()> {
    #[allow(deprecated)]
    let ix = spl_token_2022::instruction::transfer(
        ctx.program.key,
        ctx.accounts.from.key,
        ctx.accounts.to.key,
        ctx.accounts.authority.key,
        &[],
        amount,
    )?;
    solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.from.to_account_info(),
            ctx.accounts.to.to_account_info(),
            ctx.accounts.authority.to_account_info(),
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

pub fn transfer_checked<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, TransferChecked<'info>>,
    amount: u64,
    decimals: u8,
) -> Result<()> {
    let ix = spl_token_2022::instruction::transfer_checked(
        ctx.program.key,
        ctx.accounts.from.key,
        ctx.accounts.mint.key,
        ctx.accounts.to.key,
        ctx.accounts.authority.key,
        &[],
        amount,
        decimals,
    )?;
    solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.from.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.to.to_account_info(),
            ctx.accounts.authority.to_account_info(),
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

pub fn mint_to<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, MintTo<'info>>,
    amount: u64,
) -> Result<()> {
    let ix = spl_token_2022::instruction::mint_to(
        ctx.program.key,
        ctx.accounts.mint.key,
        ctx.accounts.to.key,
        ctx.accounts.authority.key,
        &[],
        amount,
    )?;
    solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.to.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.authority.to_account_info(),
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

pub fn burn<'info>(ctx: CpiContext<'_, '_, '_, 'info, Burn<'info>>, amount: u64) -> Result<()> {
    let ix = spl_token_2022::instruction::burn(
        ctx.program.key,
        ctx.accounts.from.key,
        ctx.accounts.mint.key,
        ctx.accounts.authority.key,
        &[],
        amount,
    )?;
    solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.from.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.authority.to_account_info(),
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

pub fn approve<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, Approve<'info>>,
    amount: u64,
) -> Result<()> {
    let ix = spl_token_2022::instruction::approve(
        ctx.program.key,
        ctx.accounts.to.key,
        ctx.accounts.delegate.key,
        ctx.accounts.authority.key,
        &[],
        amount,
    )?;
    solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.to.to_account_info(),
            ctx.accounts.delegate.to_account_info(),
            ctx.accounts.authority.to_account_info(),
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

pub fn revoke<'info>(ctx: CpiContext<'_, '_, '_, 'info, Revoke<'info>>) -> Result<()> {
    let ix = spl_token_2022::instruction::revoke(
        ctx.program.key,
        ctx.accounts.source.key,
        ctx.accounts.authority.key,
        &[],
    )?;
    solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.source.to_account_info(),
            ctx.accounts.authority.to_account_info(),
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

pub fn initialize_account<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, InitializeAccount<'info>>,
) -> Result<()> {
    let ix = spl_token_2022::instruction::initialize_account(
        ctx.program.key,
        ctx.accounts.account.key,
        ctx.accounts.mint.key,
        ctx.accounts.authority.key,
    )?;
    solana_program::program::invoke(
        &ix,
        &[
            ctx.accounts.account.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ],
    )
    .map_err(Into::into)
}

pub fn initialize_account3<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, InitializeAccount3<'info>>,
) -> Result<()> {
    let ix = spl_token_2022::instruction::initialize_account3(
        ctx.program.key,
        ctx.accounts.account.key,
        ctx.accounts.mint.key,
        ctx.accounts.authority.key,
    )?;
    solana_program::program::invoke(
        &ix,
        &[
            ctx.accounts.account.to_account_info(),
            ctx.accounts.mint.to_account_info(),
        ],
    )
    .map_err(Into::into)
}

pub fn close_account<'info>(ctx: CpiContext<'_, '_, '_, 'info, CloseAccount<'info>>) -> Result<()> {
    let ix = spl_token_2022::instruction::close_account(
        ctx.program.key,
        ctx.accounts.account.key,
        ctx.accounts.destination.key,
        ctx.accounts.authority.key,
        &[], // TODO: support multisig
    )?;
    solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.account.to_account_info(),
            ctx.accounts.destination.to_account_info(),
            ctx.accounts.authority.to_account_info(),
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

pub fn freeze_account<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, FreezeAccount<'info>>,
) -> Result<()> {
    let ix = spl_token_2022::instruction::freeze_account(
        ctx.program.key,
        ctx.accounts.account.key,
        ctx.accounts.mint.key,
        ctx.accounts.authority.key,
        &[], // TODO: Support multisig signers.
    )?;
    solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.account.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.authority.to_account_info(),
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

pub fn thaw_account<'info>(ctx: CpiContext<'_, '_, '_, 'info, ThawAccount<'info>>) -> Result<()> {
    let ix = spl_token_2022::instruction::thaw_account(
        ctx.program.key,
        ctx.accounts.account.key,
        ctx.accounts.mint.key,
        ctx.accounts.authority.key,
        &[], // TODO: Support multisig signers.
    )?;
    solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.account.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.authority.to_account_info(),
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

pub fn initialize_mint<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, InitializeMint<'info>>,
    decimals: u8,
    authority: &Pubkey,
    freeze_authority: Option<&Pubkey>,
) -> Result<()> {
    let ix = spl_token_2022::instruction::initialize_mint(
        ctx.program.key,
        ctx.accounts.mint.key,
        authority,
        freeze_authority,
        decimals,
    )?;
    solana_program::program::invoke(
        &ix,
        &[
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ],
    )
    .map_err(Into::into)
}

pub fn initialize_mint2<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, InitializeMint2<'info>>,
    decimals: u8,
    authority: &Pubkey,
    freeze_authority: Option<&Pubkey>,
) -> Result<()> {
    let ix = spl_token_2022::instruction::initialize_mint2(
        ctx.program.key,
        ctx.accounts.mint.key,
        authority,
        freeze_authority,
        decimals,
    )?;
    solana_program::program::invoke(&ix, &[ctx.accounts.mint.to_account_info()]).map_err(Into::into)
}

pub fn set_authority<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, SetAuthority<'info>>,
    authority_type: spl_token_2022::instruction::AuthorityType,
    new_authority: Option<Pubkey>,
) -> Result<()> {
    let mut spl_new_authority: Option<&Pubkey> = None;
    if new_authority.is_some() {
        spl_new_authority = new_authority.as_ref()
    }

    let ix = spl_token_2022::instruction::set_authority(
        ctx.program.key,
        ctx.accounts.account_or_mint.key,
        spl_new_authority,
        authority_type,
        ctx.accounts.current_authority.key,
        &[], // TODO: Support multisig signers.
    )?;
    solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.account_or_mint.to_account_info(),
            ctx.accounts.current_authority.to_account_info(),
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

pub fn sync_native<'info>(ctx: CpiContext<'_, '_, '_, 'info, SyncNative<'info>>) -> Result<()> {
    let ix = spl_token_2022::instruction::sync_native(ctx.program.key, ctx.accounts.account.key)?;
    solana_program::program::invoke(&ix, &[ctx.accounts.account.to_account_info()])
        .map_err(Into::into)
}

pub fn get_account_data_size<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, GetAccountDataSize<'info>>,
    extension_types: &[spl_token_2022::extension::ExtensionType],
) -> Result<u64> {
    let ix = spl_token_2022::instruction::get_account_data_size(
        ctx.program.key,
        ctx.accounts.mint.key,
        extension_types,
    )?;
    solana_program::program::invoke(&ix, &[ctx.accounts.mint.to_account_info()])?;
    solana_program::program::get_return_data()
        .ok_or(solana_program::program_error::ProgramError::InvalidInstructionData)
        .and_then(|(key, data)| {
            if key != *ctx.program.key {
                Err(solana_program::program_error::ProgramError::IncorrectProgramId)
            } else {
                data.try_into().map(u64::from_le_bytes).map_err(|_| {
                    solana_program::program_error::ProgramError::InvalidInstructionData
                })
            }
        })
        .map_err(Into::into)
}

pub fn initialize_mint_close_authority<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, InitializeMintCloseAuthority<'info>>,
    close_authority: Option<&Pubkey>,
) -> Result<()> {
    let ix = spl_token_2022::instruction::initialize_mint_close_authority(
        ctx.program.key,
        ctx.accounts.mint.key,
        close_authority,
    )?;
    solana_program::program::invoke(&ix, &[ctx.accounts.mint.to_account_info()]).map_err(Into::into)
}

pub fn initialize_immutable_owner<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, InitializeImmutableOwner<'info>>,
) -> Result<()> {
    let ix = spl_token_2022::instruction::initialize_immutable_owner(
        ctx.program.key,
        ctx.accounts.account.key,
    )?;
    solana_program::program::invoke(&ix, &[ctx.accounts.account.to_account_info()])
        .map_err(Into::into)
}

pub fn amount_to_ui_amount<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, AmountToUiAmount<'info>>,
    amount: u64,
) -> Result<String> {
    let ix = spl_token_2022::instruction::amount_to_ui_amount(
        ctx.program.key,
        ctx.accounts.account.key,
        amount,
    )?;
    solana_program::program::invoke(&ix, &[ctx.accounts.account.to_account_info()])?;
    solana_program::program::get_return_data()
        .ok_or(solana_program::program_error::ProgramError::InvalidInstructionData)
        .and_then(|(key, data)| {
            if key != *ctx.program.key {
                Err(solana_program::program_error::ProgramError::IncorrectProgramId)
            } else {
                String::from_utf8(data).map_err(|_| {
                    solana_program::program_error::ProgramError::InvalidInstructionData
                })
            }
        })
        .map_err(Into::into)
}

pub fn ui_amount_to_amount<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, UiAmountToAmount<'info>>,
    ui_amount: &str,
) -> Result<u64> {
    let ix = spl_token_2022::instruction::ui_amount_to_amount(
        ctx.program.key,
        ctx.accounts.account.key,
        ui_amount,
    )?;
    solana_program::program::invoke(&ix, &[ctx.accounts.account.to_account_info()])?;
    solana_program::program::get_return_data()
        .ok_or(solana_program::program_error::ProgramError::InvalidInstructionData)
        .and_then(|(key, data)| {
            if key != *ctx.program.key {
                Err(solana_program::program_error::ProgramError::IncorrectProgramId)
            } else {
                data.try_into().map(u64::from_le_bytes).map_err(|_| {
                    solana_program::program_error::ProgramError::InvalidInstructionData
                })
            }
        })
        .map_err(Into::into)
}

#[derive(Accounts)]
pub struct Transfer<'info> {
    pub from: UncheckedAccount<'info>,
    pub to: UncheckedAccount<'info>,
    pub authority: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct TransferChecked<'info> {
    pub from: UncheckedAccount<'info>,
    pub mint: UncheckedAccount<'info>,
    pub to: UncheckedAccount<'info>,
    pub authority: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct MintTo<'info> {
    pub mint: UncheckedAccount<'info>,
    pub to: UncheckedAccount<'info>,
    pub authority: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct Burn<'info> {
    pub mint: UncheckedAccount<'info>,
    pub from: UncheckedAccount<'info>,
    pub authority: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct Approve<'info> {
    pub to: UncheckedAccount<'info>,
    pub delegate: UncheckedAccount<'info>,
    pub authority: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct Revoke<'info> {
    pub source: UncheckedAccount<'info>,
    pub authority: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct InitializeAccount<'info> {
    pub account: UncheckedAccount<'info>,
    pub mint: UncheckedAccount<'info>,
    pub authority: UncheckedAccount<'info>,
    pub rent: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct InitializeAccount3<'info> {
    pub account: UncheckedAccount<'info>,
    pub mint: UncheckedAccount<'info>,
    pub authority: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct CloseAccount<'info> {
    pub account: UncheckedAccount<'info>,
    pub destination: UncheckedAccount<'info>,
    pub authority: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct FreezeAccount<'info> {
    pub account: UncheckedAccount<'info>,
    pub mint: UncheckedAccount<'info>,
    pub authority: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct ThawAccount<'info> {
    pub account: UncheckedAccount<'info>,
    pub mint: UncheckedAccount<'info>,
    pub authority: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct InitializeMint<'info> {
    pub mint: UncheckedAccount<'info>,
    pub rent: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct InitializeMint2<'info> {
    pub mint: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct SetAuthority<'info> {
    pub current_authority: UncheckedAccount<'info>,
    pub account_or_mint: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct SyncNative<'info> {
    pub account: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct GetAccountDataSize<'info> {
    pub mint: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct InitializeMintCloseAuthority<'info> {
    pub mint: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct InitializeImmutableOwner<'info> {
    pub account: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct AmountToUiAmount<'info> {
    pub account: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct UiAmountToAmount<'info> {
    pub account: UncheckedAccount<'info>,
}

#[derive(Clone)]
pub struct Token2022;

impl anchor_lang::Id for Token2022 {
    fn id() -> Pubkey {
        ID
    }
}

// Field parsers to save compute. All account validation is assumed to be done
// outside of these methods.
pub use crate::token::accessor;
