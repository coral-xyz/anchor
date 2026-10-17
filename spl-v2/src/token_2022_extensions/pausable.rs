use {
    super::common::validate_token_2022_program,
    crate::token_shared::token_data_subinstruction,
    anchor_lang::{CpiContext, CpiHandle, CpiHandleMut, ToCpiAccounts},
    pinocchio::address::Address,
    solana_program_error::ProgramError,
};

#[derive(ToCpiAccounts)]
pub struct PausableInitialize<'a> {
    pub mint: CpiHandleMut<'a>,
}

#[derive(ToCpiAccounts)]
pub struct PausableToggle<'a> {
    pub mint: CpiHandleMut<'a>,
    #[signer]
    pub authority: CpiHandle<'a>,
}

pub fn pausable_initialize<'a>(
    ctx: CpiContext<'a, PausableInitialize<'a>>,
    authority: &Address,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let mut data = token_data_subinstruction(44, 0)?;
    crate::token_shared::extend_token_data(&mut data, authority.as_ref())?;
    ctx.invoke(&data)
}

pub fn pausable_pause<'a>(ctx: CpiContext<'a, PausableToggle<'a>>) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let data = token_data_subinstruction(44, 1)?;
    ctx.invoke_with_signer_override(&data, Some((1, ctx.remaining_accounts.is_empty())))
}

pub fn pausable_resume<'a>(ctx: CpiContext<'a, PausableToggle<'a>>) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let data = token_data_subinstruction(44, 2)?;
    ctx.invoke_with_signer_override(&data, Some((1, ctx.remaining_accounts.is_empty())))
}
