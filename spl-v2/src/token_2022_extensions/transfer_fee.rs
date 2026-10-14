use {
    super::common::validate_token_2022_program,
    crate::{
        token_shared::{
            push_coption_address, push_remaining_account, token_data_subinstruction,
            TokenInstructionData,
        },
    },
    anchor_lang::{CpiContext, CpiHandle, CpiHandleMut, ToCpiAccounts},
    pinocchio::address::Address,
    solana_program_error::ProgramError,
};

#[derive(ToCpiAccounts)]
pub struct TransferFeeInitialize<'a> {
    pub mint: CpiHandleMut<'a>,
}

#[derive(ToCpiAccounts)]
pub struct TransferFeeSetTransferFee<'a> {
    pub mint: CpiHandleMut<'a>,
    #[signer]
    pub authority: CpiHandle<'a>,
}

#[derive(ToCpiAccounts)]
pub struct TransferCheckedWithFee<'a> {
    pub source: CpiHandleMut<'a>,
    pub mint: CpiHandle<'a>,
    pub destination: CpiHandleMut<'a>,
    #[signer]
    pub authority: CpiHandle<'a>,
}

#[derive(ToCpiAccounts)]
pub struct HarvestWithheldTokensToMint<'a> {
    pub mint: CpiHandleMut<'a>,
}

#[derive(ToCpiAccounts)]
pub struct WithdrawWithheldTokensFromMint<'a> {
    pub mint: CpiHandleMut<'a>,
    pub destination: CpiHandleMut<'a>,
    #[signer]
    pub authority: CpiHandle<'a>,
}

#[derive(ToCpiAccounts)]
pub struct WithdrawWithheldTokensFromAccounts<'a> {
    pub mint: CpiHandle<'a>,
    pub destination: CpiHandleMut<'a>,
    #[signer]
    pub authority: CpiHandle<'a>,
}

pub fn transfer_fee_initialize<'a>(
    ctx: CpiContext<'a, TransferFeeInitialize<'a>>,
    transfer_fee_config_authority: Option<&Address>,
    withdraw_withheld_authority: Option<&Address>,
    transfer_fee_basis_points: u16,
    maximum_fee: u64,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let mut data: TokenInstructionData = token_data_subinstruction(26, 0);
    push_coption_address(&mut data, transfer_fee_config_authority);
    push_coption_address(&mut data, withdraw_withheld_authority);
    data.extend_from_slice(&transfer_fee_basis_points.to_le_bytes());
    data.extend_from_slice(&maximum_fee.to_le_bytes());
    ctx.invoke(&data)
}

pub fn transfer_fee_set<'a>(
    ctx: CpiContext<'a, TransferFeeSetTransferFee<'a>>,
    transfer_fee_basis_points: u16,
    maximum_fee: u64,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let mut data = token_data_subinstruction(26, 5);
    data.extend_from_slice(&transfer_fee_basis_points.to_le_bytes());
    data.extend_from_slice(&maximum_fee.to_le_bytes());
    ctx.invoke_with_signer_override(&data, Some((1, ctx.remaining_accounts.is_empty())))
}

pub fn transfer_checked_with_fee<'a>(
    ctx: CpiContext<'a, TransferCheckedWithFee<'a>>,
    amount: u64,
    decimals: u8,
    fee: u64,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let mut data = token_data_subinstruction(26, 1);
    data.extend_from_slice(&amount.to_le_bytes());
    data.push(decimals);
    data.extend_from_slice(&fee.to_le_bytes());
    ctx.invoke_with_signer_override(&data, Some((3, ctx.remaining_accounts.is_empty())))
}

pub fn harvest_withheld_tokens_to_mint<'a>(
    ctx: CpiContext<'a, HarvestWithheldTokensToMint<'a>>,
    sources: &[CpiHandleMut<'a>],
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let mut remaining_accounts: anchor_lang::CpiHandleBuffer<'a> = Default::default();
    for source in sources {
        push_remaining_account(&mut remaining_accounts, (*source).into())?;
    }
    for account in ctx.remaining_accounts.iter().copied() {
        push_remaining_account(&mut remaining_accounts, account)?;
    }
    let data = token_data_subinstruction(26, 4);
    #[cfg(not(target_os = "solana"))]
    return ctx.with_remaining_accounts(remaining_accounts).invoke(&data);
    #[cfg(target_os = "solana")]
    ctx.with_remaining_accounts(remaining_accounts.as_slice())
        .invoke(&data)
}

pub fn withdraw_withheld_tokens_from_mint<'a>(
    ctx: CpiContext<'a, WithdrawWithheldTokensFromMint<'a>>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let data = token_data_subinstruction(26, 2);
    ctx.invoke_with_signer_override(&data, Some((2, ctx.remaining_accounts.is_empty())))
}

pub fn withdraw_withheld_tokens_from_accounts<'a>(
    ctx: CpiContext<'a, WithdrawWithheldTokensFromAccounts<'a>>,
    sources: &[CpiHandleMut<'a>],
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let has_signers = !ctx.remaining_accounts.is_empty();
    let mut remaining_accounts = anchor_lang::CpiHandleBuffer::new();
    for account in ctx.remaining_accounts.iter().copied() {
        push_remaining_account(&mut remaining_accounts, account)?;
    }
    for source in sources {
        push_remaining_account(&mut remaining_accounts, CpiHandle::from(*source))?;
    }
    let mut data = token_data_subinstruction(26, 3);
    data.push(sources.len() as u8);
    #[cfg(not(target_os = "solana"))]
    return ctx
        .with_remaining_accounts(remaining_accounts)
        .invoke_with_signer_override(&data, Some((2, !has_signers)));
    #[cfg(target_os = "solana")]
    ctx.with_remaining_accounts(remaining_accounts.as_slice())
        .invoke_with_signer_override(&data, Some((2, !has_signers)))
}
