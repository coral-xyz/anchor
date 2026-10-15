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
pub struct MetadataPointerInitialize<'a> {
    pub mint: CpiHandleMut<'a>,
}

#[derive(ToCpiAccounts)]
pub struct MetadataPointerUpdate<'a> {
    pub mint: CpiHandleMut<'a>,
    #[signer]
    pub authority: CpiHandle<'a>,
}

pub fn metadata_pointer_initialize<'a>(
    ctx: CpiContext<'a, MetadataPointerInitialize<'a>>,
    authority: Option<&Address>,
    metadata_address: Option<&Address>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let mut data: TokenInstructionData = token_data_subinstruction(39, 0)?;
    push_optional_zero_address(&mut data, authority)?;
    push_optional_zero_address(&mut data, metadata_address)?;
    ctx.invoke(&data)
}

pub fn metadata_pointer_update<'a>(
    ctx: CpiContext<'a, MetadataPointerUpdate<'a>>,
    metadata_address: Option<&Address>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let mut data = token_data_subinstruction(39, 1)?;
    push_optional_zero_address(&mut data, metadata_address)?;
    ctx.invoke_with_signer_override(&data, Some((1, ctx.remaining_accounts.is_empty())))
}
