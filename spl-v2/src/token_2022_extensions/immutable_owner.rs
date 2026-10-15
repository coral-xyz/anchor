use {
    super::common::validate_token_2022_program,
    crate::token_shared::token_data,
    anchor_lang::{CpiContext, CpiHandleMut, ToCpiAccounts},
    solana_program_error::ProgramError,
};

#[derive(ToCpiAccounts)]
pub struct ImmutableOwnerInitialize<'a> {
    pub token_account: CpiHandleMut<'a>,
}

pub fn immutable_owner_initialize<'a>(
    ctx: CpiContext<'a, ImmutableOwnerInitialize<'a>>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let data = token_data(22)?;
    ctx.invoke(&data)
}
