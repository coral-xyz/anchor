use {
    super::common::validate_token_2022_program,
    crate::{
        token_shared::{
            push_optional_zero_address, token_data_subinstruction,
            TokenInstructionData,
        },
    },
    anchor_lang::{CpiContext, CpiHandle, CpiHandleMut, ToCpiAccounts},
    pinocchio::address::Address,
    solana_program_error::ProgramError,
};

#[derive(ToCpiAccounts)]
pub struct TransferHookInitialize<'a> {
    pub mint: CpiHandleMut<'a>,
}

#[derive(ToCpiAccounts)]
pub struct TransferHookUpdate<'a> {
    pub mint: CpiHandleMut<'a>,
    #[signer]
    pub authority: CpiHandle<'a>,
}

pub fn transfer_hook_initialize<'a>(
    ctx: CpiContext<'a, TransferHookInitialize<'a>>,
    authority: Option<&Address>,
    transfer_hook_program_id: Option<&Address>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let mut data: TokenInstructionData = token_data_subinstruction(36, 0)?;
    push_optional_zero_address(&mut data, authority)?;
    push_optional_zero_address(&mut data, transfer_hook_program_id)?;
    ctx.invoke(&data)
}

pub fn transfer_hook_update<'a>(
    ctx: CpiContext<'a, TransferHookUpdate<'a>>,
    transfer_hook_program_id: Option<&Address>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let mut data = token_data_subinstruction(36, 1)?;
    push_optional_zero_address(&mut data, transfer_hook_program_id)?;
    ctx.invoke_with_signer_override(&data, Some((1, ctx.remaining_accounts.is_empty())))
}
