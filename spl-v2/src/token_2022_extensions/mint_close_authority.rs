use {
    super::common::validate_token_2022_program,
    crate::token_shared::token_data_option_address,
    anchor_lang::{CpiContext, CpiHandleMut, ToCpiAccounts},
    pinocchio::address::Address,
    solana_program_error::ProgramError,
};

#[derive(ToCpiAccounts)]
pub struct MintCloseAuthorityInitialize<'a> {
    pub mint: CpiHandleMut<'a>,
}

pub fn mint_close_authority_initialize<'a>(
    ctx: CpiContext<'a, MintCloseAuthorityInitialize<'a>>,
    close_authority: Option<&Address>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let data = token_data_option_address(25, close_authority);
    ctx.invoke(&data)
}
