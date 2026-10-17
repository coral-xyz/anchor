use {
    super::common::validate_token_2022_program,
    crate::token_shared::{extend_token_data, push_optional_zero_address, TokenInstructionData},
    anchor_lang::{CpiContext, CpiHandle, CpiHandleMut, ToCpiAccounts},
    pinocchio::address::Address,
    solana_program_error::ProgramError,
};

const INITIALIZE_GROUP_DISCRIMINATOR: [u8; 8] =
    [0x79, 0x71, 0x6c, 0x27, 0x36, 0x33, 0x00, 0x04];
const INITIALIZE_MEMBER_DISCRIMINATOR: [u8; 8] =
    [0x98, 0x20, 0xde, 0xb0, 0xdf, 0xed, 0x74, 0x86];

#[derive(ToCpiAccounts)]
pub struct TokenGroupInitialize<'a> {
    pub group: CpiHandleMut<'a>,
    pub mint: CpiHandle<'a>,
    #[signer]
    pub mint_authority: CpiHandle<'a>,
}

#[derive(ToCpiAccounts)]
pub struct TokenMemberInitialize<'a> {
    pub member: CpiHandleMut<'a>,
    pub member_mint: CpiHandle<'a>,
    #[signer]
    pub member_mint_authority: CpiHandle<'a>,
    pub group: CpiHandleMut<'a>,
    #[signer]
    pub group_update_authority: CpiHandle<'a>,
}

pub fn token_group_initialize<'a>(
    ctx: CpiContext<'a, TokenGroupInitialize<'a>>,
    update_authority: Option<&Address>,
    max_size: u64,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let mut data = TokenInstructionData::new();
    extend_token_data(&mut data, &INITIALIZE_GROUP_DISCRIMINATOR)?;
    push_optional_zero_address(&mut data, update_authority)?;
    extend_token_data(&mut data, &max_size.to_le_bytes())?;
    ctx.invoke(&data)
}

pub fn token_member_initialize<'a>(
    ctx: CpiContext<'a, TokenMemberInitialize<'a>>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    ctx.invoke(&INITIALIZE_MEMBER_DISCRIMINATOR)
}
