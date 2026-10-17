//! Token-2022 CPI helpers and interface re-exports.

use {
    crate::token_shared::{
        extend_token_data, token_data, token_data_address,
        token_data_option_address,
    },
    crate::token_2022_extensions::common::validate_token_2022_program,
    anchor_lang::{require_eq, CpiContext, CpiHandle, CpiHandleMut, ToCpiAccounts},
    pinocchio::address::Address,
    solana_program_error::ProgramError,
};

pub use anchor_lang::programs::Token2022;
pub use crate::protocol::token_2022::{
    self as spl_token_2022, extension::ExtensionType, native_mint, ID,
};

pub use crate::token_shared::{
    approve, approve_checked, burn, burn_checked, close_account, freeze_account,
    initialize_account, initialize_account3, initialize_mint, initialize_mint2, mint_to,
    mint_to_checked, revoke, set_authority, sync_native, thaw_account, transfer, transfer_checked,
    Approve, ApproveChecked, Burn, BurnChecked, CloseAccount, FreezeAccount, InitializeAccount,
    InitializeAccount3, InitializeMint, InitializeMint2, MintTo, MintToChecked, Revoke,
    SetAuthority, SyncNative, ThawAccount, Transfer, TransferChecked,
};

#[derive(ToCpiAccounts)]
pub struct GetAccountDataSize<'a> {
    pub mint: CpiHandle<'a>,
}

#[derive(ToCpiAccounts)]
pub struct InitializeMintCloseAuthority<'a> {
    pub mint: CpiHandleMut<'a>,
}

#[derive(ToCpiAccounts)]
pub struct InitializeImmutableOwner<'a> {
    pub account: CpiHandleMut<'a>,
}

#[derive(ToCpiAccounts)]
pub struct AmountToUiAmount<'a> {
    pub account: CpiHandle<'a>,
}

#[derive(ToCpiAccounts)]
pub struct UiAmountToAmount<'a> {
    pub account: CpiHandle<'a>,
}

#[derive(ToCpiAccounts)]
pub struct Reallocate<'a> {
    pub account: CpiHandleMut<'a>,
    #[signer]
    pub payer: CpiHandleMut<'a>,
    pub system_program: CpiHandle<'a>,
    #[signer]
    pub authority: CpiHandle<'a>,
}

#[derive(ToCpiAccounts)]
pub struct WithdrawExcessLamports<'a> {
    pub source: CpiHandleMut<'a>,
    pub destination: CpiHandleMut<'a>,
    #[signer]
    pub authority: CpiHandle<'a>,
}

#[derive(ToCpiAccounts)]
pub struct CreateNativeMint<'a> {
    #[signer]
    pub payer: CpiHandleMut<'a>,
    pub native_mint: CpiHandleMut<'a>,
    pub system_program: CpiHandle<'a>,
}

#[derive(ToCpiAccounts)]
pub struct InitializeNonTransferableMint<'a> {
    pub mint: CpiHandleMut<'a>,
}

#[derive(ToCpiAccounts)]
pub struct PermanentDelegateInitialize<'a> {
    pub mint: CpiHandleMut<'a>,
}

#[cfg(not(any(target_os = "solana", target_arch = "bpf")))]
#[inline(never)]
fn return_data_from(program: &Address) -> Result<std::vec::Vec<u8>, ProgramError> {
    let (return_program, data) = anchor_lang::solana_program::program::get_return_data()
        .ok_or(ProgramError::InvalidInstructionData)?;
    require_eq!(
        return_program.to_bytes().as_slice(),
        program.as_ref(),
        ProgramError::IncorrectProgramId
    );
    Ok(data.as_slice().to_vec())
}

#[cfg(any(target_os = "solana", target_arch = "bpf"))]
#[inline(never)]
fn return_data_from(
    program: &Address,
) -> Result<anchor_lang::FixedVec<u8, { anchor_lang::MAX_INSTRUCTION_DATA }>, ProgramError> {
    let (return_program, data) = anchor_lang::solana_program::program::get_return_data()
        .ok_or(ProgramError::InvalidInstructionData)?;
    require_eq!(
        return_program.to_bytes().as_slice(),
        program.as_ref(),
        ProgramError::IncorrectProgramId
    );
    let mut out = anchor_lang::FixedVec::new();
    out.try_extend_from_slice(data.as_slice())
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    Ok(out)
}

#[inline(never)]
fn return_u64_from(program: &Address) -> Result<u64, ProgramError> {
    let (return_program, data) = anchor_lang::solana_program::program::get_return_data()
        .ok_or(ProgramError::InvalidInstructionData)?;
    require_eq!(
        return_program.to_bytes().as_slice(),
        program.as_ref(),
        ProgramError::IncorrectProgramId
    );
    let bytes: [u8; 8] = data
        .as_slice()
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    Ok(u64::from_le_bytes(bytes))
}

pub fn get_account_data_size<'a>(
    ctx: CpiContext<'a, GetAccountDataSize<'a>>,
    extension_types: &[ExtensionType],
) -> Result<u64, ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let mut data = token_data(21)?;
    for extension_type in extension_types {
        extend_token_data(&mut data, &<[u8; 2]>::from(*extension_type))?;
    }
    ctx.invoke(&data)?;
    drop(data);
    return_u64_from(ctx.program)
}

pub fn initialize_mint_close_authority<'a>(
    ctx: CpiContext<'a, InitializeMintCloseAuthority<'a>>,
    close_authority: Option<&Address>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let data = token_data_option_address(25, close_authority)?;
    ctx.invoke(&data)
}

pub fn initialize_immutable_owner<'a>(
    ctx: CpiContext<'a, InitializeImmutableOwner<'a>>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let data = token_data(22)?;
    ctx.invoke(&data)
}

#[cfg(not(any(target_os = "solana", target_arch = "bpf")))]
pub fn amount_to_ui_amount<'a>(
    ctx: CpiContext<'a, AmountToUiAmount<'a>>,
    amount: u64,
) -> Result<std::string::String, ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let data = crate::token_shared::token_data_u64(23, amount)?;
    ctx.invoke(&data)?;
    let bytes = return_data_from(ctx.program)?;
    std::string::String::from_utf8(bytes).map_err(|_| ProgramError::InvalidInstructionData)
}

#[cfg(any(target_os = "solana", target_arch = "bpf"))]
pub fn amount_to_ui_amount<'a>(
    ctx: CpiContext<'a, AmountToUiAmount<'a>>,
    amount: u64,
) -> Result<anchor_lang::FixedVec<u8, { anchor_lang::MAX_INSTRUCTION_DATA }>, ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let data = crate::token_shared::token_data_u64(23, amount)?;
    ctx.invoke(&data)?;
    drop(data);
    Ok(return_data_from(ctx.program)?)
}

pub fn ui_amount_to_amount<'a>(
    ctx: CpiContext<'a, UiAmountToAmount<'a>>,
    ui_amount: &str,
) -> Result<u64, ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let mut data = token_data(24)?;
    extend_token_data(&mut data, ui_amount.as_bytes())?;
    ctx.invoke(&data)?;
    drop(data);
    return_u64_from(ctx.program)
}

pub fn reallocate<'a>(
    ctx: CpiContext<'a, Reallocate<'a>>,
    extension_types: &[ExtensionType],
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let mut data = token_data(29)?;
    for extension_type in extension_types {
        extend_token_data(&mut data, &<[u8; 2]>::from(*extension_type))?;
    }
    ctx.invoke_with_signer_override(&data, Some((3, ctx.remaining_accounts.len() == 0)))
}

pub fn withdraw_excess_lamports<'a>(
    ctx: CpiContext<'a, WithdrawExcessLamports<'a>>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let data = token_data(38)?;
    ctx.invoke_with_signer_override(&data, Some((2, ctx.remaining_accounts.len() == 0)))
}

pub fn create_native_mint<'a>(
    ctx: CpiContext<'a, CreateNativeMint<'a>>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let data = token_data(31)?;
    ctx.invoke(&data)
}

pub fn initialize_non_transferable_mint<'a>(
    ctx: CpiContext<'a, InitializeNonTransferableMint<'a>>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let data = token_data(32)?;
    ctx.invoke(&data)
}

pub fn initialize_permanent_delegate<'a>(
    ctx: CpiContext<'a, PermanentDelegateInitialize<'a>>,
    permanent_delegate: &Address,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let data = token_data_address(35, permanent_delegate)?;
    ctx.invoke(&data)
}
