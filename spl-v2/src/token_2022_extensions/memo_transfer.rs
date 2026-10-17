use {
    super::common::validate_token_2022_program,
    crate::token_shared::token_data_subinstruction,
    anchor_lang::{CpiContext, CpiHandle, CpiHandleMut, ToCpiAccounts},
    solana_program_error::ProgramError,
};

#[derive(ToCpiAccounts)]
pub struct MemoTransfer<'a> {
    pub account: CpiHandleMut<'a>,
    #[signer]
    pub owner: CpiHandle<'a>,
}

pub fn memo_transfer_initialize<'a>(
    ctx: CpiContext<'a, MemoTransfer<'a>>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let data = token_data_subinstruction(30, 0)?;
    ctx.invoke_with_signer_override(&data, Some((1, ctx.remaining_accounts.is_empty())))
}

pub fn memo_transfer_disable<'a>(
    ctx: CpiContext<'a, MemoTransfer<'a>>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let data = token_data_subinstruction(30, 1)?;
    ctx.invoke_with_signer_override(&data, Some((1, ctx.remaining_accounts.is_empty())))
}
