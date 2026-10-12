use {
    super::common::validate_token_2022_program,
    crate::token_shared::{extend_token_data, push_optional_zero_address, TokenInstructionData},
    anchor_lang::{CpiContext, CpiHandle, CpiHandleMut, ToCpiAccounts},
    pinocchio::address::Address,
    solana_program_error::ProgramError,
};

pub use crate::protocol::token_metadata::Field;

const INITIALIZE_DISCRIMINATOR: [u8; 8] = [0xd2, 0xe1, 0x1e, 0xa2, 0x58, 0xb8, 0x4d, 0x8d];
const UPDATE_FIELD_DISCRIMINATOR: [u8; 8] = [0xdd, 0xe9, 0x31, 0x2d, 0xb5, 0xca, 0xdc, 0xc8];
const REMOVE_KEY_DISCRIMINATOR: [u8; 8] = [0xea, 0x12, 0x20, 0x38, 0x59, 0x8d, 0x25, 0xb5];
const UPDATE_AUTHORITY_DISCRIMINATOR: [u8; 8] = [0xd7, 0xe4, 0xa6, 0xe4, 0x54, 0x64, 0x56, 0x7b];

fn push_string(data: &mut TokenInstructionData, value: &str) -> Result<(), ProgramError> {
    extend_token_data(data, &(value.len() as u32).to_le_bytes())?;
    extend_token_data(data, value.as_bytes())
}

#[derive(ToCpiAccounts)]
pub struct TokenMetadataInitialize<'a> {
    pub metadata: CpiHandleMut<'a>,
    pub update_authority: CpiHandle<'a>,
    pub mint: CpiHandle<'a>,
    #[signer]
    pub mint_authority: CpiHandle<'a>,
}

#[derive(ToCpiAccounts)]
pub struct TokenMetadataUpdateAuthority<'a> {
    pub metadata: CpiHandleMut<'a>,
    #[signer]
    pub current_authority: CpiHandle<'a>,
}

#[derive(ToCpiAccounts)]
pub struct TokenMetadataUpdateField<'a> {
    pub metadata: CpiHandleMut<'a>,
    #[signer]
    pub update_authority: CpiHandle<'a>,
}

#[derive(ToCpiAccounts)]
pub struct TokenMetadataRemoveKey<'a> {
    pub metadata: CpiHandleMut<'a>,
    #[signer]
    pub update_authority: CpiHandle<'a>,
}

pub fn token_metadata_initialize<'a>(
    ctx: CpiContext<'a, TokenMetadataInitialize<'a>>,
    name: &str,
    symbol: &str,
    uri: &str,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let mut data = TokenInstructionData::new();
    data.extend_from_slice(&INITIALIZE_DISCRIMINATOR);
    push_string(&mut data, name)?;
    push_string(&mut data, symbol)?;
    push_string(&mut data, uri)?;
    ctx.invoke(&data)
}

pub fn token_metadata_update_authority<'a>(
    ctx: CpiContext<'a, TokenMetadataUpdateAuthority<'a>>,
    new_authority: Option<&Address>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let mut data = TokenInstructionData::new();
    data.extend_from_slice(&UPDATE_AUTHORITY_DISCRIMINATOR);
    push_optional_zero_address(&mut data, new_authority);
    ctx.invoke(&data)
}

pub fn token_metadata_update_field<'a>(
    ctx: CpiContext<'a, TokenMetadataUpdateField<'a>>,
    field: Field,
    value: &str,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let mut data = TokenInstructionData::new();
    data.extend_from_slice(&UPDATE_FIELD_DISCRIMINATOR);
    match field {
        Field::Name => data.push(0),
        Field::Symbol => data.push(1),
        Field::Uri => data.push(2),
        Field::Key(key) => {
            data.push(3);
            push_string(&mut data, &key)?;
        }
    }
    push_string(&mut data, value)?;
    ctx.invoke(&data)
}

pub fn token_metadata_remove_key<'a>(
    ctx: CpiContext<'a, TokenMetadataRemoveKey<'a>>,
    key: &str,
    idempotent: bool,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let mut data = TokenInstructionData::new();
    data.extend_from_slice(&REMOVE_KEY_DISCRIMINATOR);
    data.push(idempotent as u8);
    push_string(&mut data, key)?;
    ctx.invoke(&data)
}
