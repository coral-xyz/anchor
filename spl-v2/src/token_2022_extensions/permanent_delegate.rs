use {
    super::common::validate_token_2022_program,
    crate::token_shared::token_data_address,
    anchor_lang::{CpiContext, CpiHandleMut, ToCpiAccounts},
    pinocchio::address::Address,
    solana_program_error::ProgramError,
};

#[derive(ToCpiAccounts)]
pub struct PermanentDelegateInitialize<'a> {
    pub mint: CpiHandleMut<'a>,
}

pub fn permanent_delegate_initialize<'a>(
    ctx: CpiContext<'a, PermanentDelegateInitialize<'a>>,
    permanent_delegate: &Address,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let data = token_data_address(35, permanent_delegate);
    ctx.invoke(&data)
}
