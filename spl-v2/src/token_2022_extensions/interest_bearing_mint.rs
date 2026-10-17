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
pub struct InterestBearingMintInitialize<'a> {
    pub mint: CpiHandleMut<'a>,
}

#[derive(ToCpiAccounts)]
pub struct InterestBearingMintUpdateRate<'a> {
    pub mint: CpiHandleMut<'a>,
    #[signer]
    pub rate_authority: CpiHandle<'a>,
}

pub fn interest_bearing_mint_initialize<'a>(
    ctx: CpiContext<'a, InterestBearingMintInitialize<'a>>,
    rate_authority: Option<&Address>,
    rate: i16,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let mut data: TokenInstructionData = token_data_subinstruction(33, 0)?;
    push_optional_zero_address(&mut data, rate_authority)?;
    crate::token_shared::extend_token_data(&mut data, &rate.to_le_bytes())?;
    ctx.invoke(&data)
}

pub fn interest_bearing_mint_update_rate<'a>(
    ctx: CpiContext<'a, InterestBearingMintUpdateRate<'a>>,
    rate: i16,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let mut data = token_data_subinstruction(33, 1)?;
    crate::token_shared::extend_token_data(&mut data, &rate.to_le_bytes())?;
    ctx.invoke_with_signer_override(&data, Some((1, ctx.remaining_accounts.is_empty())))
}
