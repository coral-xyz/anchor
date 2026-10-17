use {
    super::common::validate_token_2022_program,
    crate::token_shared::token_data,
    anchor_lang::{CpiContext, CpiHandleMut, ToCpiAccounts},
    solana_program_error::ProgramError,
};

#[derive(ToCpiAccounts)]
pub struct NonTransferableMintInitialize<'a> {
    pub mint: CpiHandleMut<'a>,
}

pub fn non_transferable_mint_initialize<'a>(
    ctx: CpiContext<'a, NonTransferableMintInitialize<'a>>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let data = token_data(32)?;
    ctx.invoke(&data)
}
