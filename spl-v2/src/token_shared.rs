//! Shared base token CPI helpers used by `token` and `token_2022`.
//!
//! Authority-bearing helpers support both single authorities and SPL
//! multisigs. For a multisig, pass the member signer handles in canonical
//! order through [`CpiContext::with_remaining_accounts`].

#[cfg(feature = "guardrails")]
use anchor_lang::{require, Id};
use {
    anchor_lang::{CpiContext, CpiHandle, CpiHandleMut, ToCpiAccounts},
    pinocchio::address::Address,
    solana_program_error::ProgramError,
    crate::protocol::token_2022::{self as spl_token_2022},
};

#[cfg(test)]
use anchor_lang::FixedVec;

pub(crate) type TokenInstructionData = anchor_lang::FixedVec<u8, 256>;

#[inline]
pub(crate) fn push_remaining_account<'a>(
    accounts: &mut anchor_lang::CpiHandleBuffer<'a>,
    account: anchor_lang::CpiHandle<'a>,
) -> Result<(), ProgramError> {
    #[cfg(target_os = "solana")]
    {
        accounts
            .try_push(account)
            .map_err(|_| ProgramError::InvalidInstructionData)
    }
    #[cfg(not(target_os = "solana"))]
    {
        accounts.push(account);
        Ok(())
    }
}

#[inline]
pub(crate) fn token_data(opcode: u8) -> TokenInstructionData {
    let mut data = TokenInstructionData::new();
    data.push(opcode);
    data
}

#[inline]
pub(crate) fn token_data_u64(opcode: u8, amount: u64) -> TokenInstructionData {
    let mut data = token_data(opcode);
    data.extend_from_slice(&amount.to_le_bytes());
    data
}

#[inline]
pub(crate) fn token_data_u64_u8(
    opcode: u8,
    amount: u64,
    value: u8,
) -> TokenInstructionData {
    let mut data = token_data_u64(opcode, amount);
    data.push(value);
    data
}

#[inline]
pub(crate) fn token_data_address(
    opcode: u8,
    address: &Address,
) -> TokenInstructionData {
    let mut data = token_data(opcode);
    data.extend_from_slice(address.as_ref());
    data
}

#[inline]
pub(crate) fn token_data_option_address(
    opcode: u8,
    address: Option<&Address>,
) -> TokenInstructionData {
    let mut data = token_data(opcode);
    match address {
        Some(address) => {
            data.push(1);
            data.extend_from_slice(address.as_ref());
        }
        None => data.push(0),
    }
    data
}

#[inline]
pub(crate) fn token_data_subinstruction(opcode: u8, subinstruction: u8) -> TokenInstructionData {
    let mut data = token_data(opcode);
    data.push(subinstruction);
    data
}

#[inline]
pub(crate) fn push_optional_zero_address(
    data: &mut TokenInstructionData,
    address: Option<&Address>,
) {
    match address {
        Some(address) => data.extend_from_slice(address.as_ref()),
        None => data.extend_from_slice(&[0; 32]),
    }
}

#[inline]
pub(crate) fn push_coption_address(data: &mut TokenInstructionData, address: Option<&Address>) {
    match address {
        Some(address) => {
            data.push(1);
            data.extend_from_slice(address.as_ref());
        }
        None => data.push(0),
    }
}

#[inline]
pub(crate) fn extend_token_data(
    data: &mut TokenInstructionData,
    bytes: &[u8],
) -> Result<(), ProgramError> {
    #[cfg(target_os = "solana")]
    {
        data.try_extend_from_slice(bytes)
            .map_err(|_| ProgramError::InvalidInstructionData)
    }
    #[cfg(not(target_os = "solana"))]
    {
        data.extend_from_slice(bytes);
        Ok(())
    }
}

#[inline]
fn authority_type_byte(authority_type: &spl_token_2022::instruction::AuthorityType) -> u8 {
    use spl_token_2022::instruction::AuthorityType;
    match authority_type {
        AuthorityType::MintTokens => 0,
        AuthorityType::FreezeAccount => 1,
        AuthorityType::AccountOwner => 2,
        AuthorityType::CloseAccount => 3,
        AuthorityType::TransferFeeConfig => 4,
        AuthorityType::WithheldWithdraw => 5,
        AuthorityType::CloseMint => 6,
        AuthorityType::InterestRate => 7,
        AuthorityType::PermanentDelegate => 8,
        AuthorityType::ConfidentialTransferMint => 9,
        AuthorityType::TransferHookProgramId => 10,
        AuthorityType::ConfidentialTransferFeeConfig => 11,
        AuthorityType::MetadataPointer => 12,
        AuthorityType::GroupPointer => 13,
        AuthorityType::GroupMemberPointer => 14,
        AuthorityType::ScaledUiAmount => 15,
        AuthorityType::Pause => 16,
    }
}

#[inline]
fn token_data_set_authority(
    authority_type: &spl_token_2022::instruction::AuthorityType,
    new_authority: Option<&Address>,
) -> TokenInstructionData {
    let mut data = token_data(6);
    data.push(authority_type_byte(authority_type));
    match new_authority {
        Some(address) => {
            data.push(1);
            data.extend_from_slice(address.as_ref());
        }
        None => data.push(0),
    }
    data
}

/// SPL Token encodes a multisig authority by marking the authority account
/// non-signer and appending each member signer to the instruction. Token CPI
/// callers provide those member handles through `remaining_accounts`.
#[cfg(test)]
pub(crate) fn multisig_signer_addresses<'a>(
    accounts: &[CpiHandle<'a>],
) -> FixedVec<&'a Address, { anchor_lang::MAX_ACCOUNT_METAS }> {
    let mut addresses = FixedVec::new();
    for account in accounts {
        addresses.push(account.address());
    }
    addresses
}

#[cfg(feature = "guardrails")]
#[inline]
pub(crate) fn validate_token_interface_program(program_id: &Address) -> Result<(), ProgramError> {
    require!(
        anchor_lang::address_eq(program_id, &anchor_lang::programs::Token::id())
            || anchor_lang::address_eq(program_id, &anchor_lang::programs::Token2022::id()),
        ProgramError::IncorrectProgramId
    );
    Ok(())
}

#[cfg(not(feature = "guardrails"))]
#[inline]
pub(crate) fn validate_token_interface_program(_program_id: &Address) -> Result<(), ProgramError> {
    Ok(())
}

#[derive(ToCpiAccounts)]
pub struct InitializeAccount<'a> {
    pub account: CpiHandleMut<'a>,
    pub mint: CpiHandle<'a>,
    pub authority: CpiHandle<'a>,
    pub rent: CpiHandle<'a>,
}

#[derive(ToCpiAccounts)]
pub struct InitializeAccount3<'a> {
    pub account: CpiHandleMut<'a>,
    pub mint: CpiHandle<'a>,
    #[account_meta(skip)]
    pub authority: CpiHandle<'a>,
}

#[derive(ToCpiAccounts)]
pub struct InitializeMint<'a> {
    pub mint: CpiHandleMut<'a>,
    pub rent: CpiHandle<'a>,
}

#[derive(ToCpiAccounts)]
pub struct InitializeMint2<'a> {
    pub mint: CpiHandleMut<'a>,
}

/// Token / Token-2022 transfer instruction — accounts list:
///   0. `[writable]` from
///   1. `[writable]` to
///   2. `[signer]` authority, or `[]` multisig authority followed by member signers
#[derive(ToCpiAccounts)]
pub struct Transfer<'a> {
    pub from: CpiHandleMut<'a>,
    pub to: CpiHandleMut<'a>,
    #[signer]
    pub authority: CpiHandle<'a>,
}

/// Token / Token-2022 checked transfer instruction — adds the mint and verifies
/// the declared decimals match on-chain.
///   0. `[writable]` from
///   1. `[]` mint
///   2. `[writable]` to
///   3. `[signer]` authority, or `[]` multisig authority followed by member signers
#[derive(ToCpiAccounts)]
pub struct TransferChecked<'a> {
    pub from: CpiHandleMut<'a>,
    pub mint: CpiHandle<'a>,
    pub to: CpiHandleMut<'a>,
    #[signer]
    pub authority: CpiHandle<'a>,
}

#[derive(ToCpiAccounts)]
pub struct MintTo<'a> {
    pub mint: CpiHandleMut<'a>,
    pub to: CpiHandleMut<'a>,
    #[signer]
    pub authority: CpiHandle<'a>,
}

#[derive(ToCpiAccounts)]
pub struct MintToChecked<'a> {
    pub mint: CpiHandleMut<'a>,
    pub to: CpiHandleMut<'a>,
    #[signer]
    pub authority: CpiHandle<'a>,
}

#[derive(ToCpiAccounts)]
pub struct Burn<'a> {
    pub from: CpiHandleMut<'a>,
    pub mint: CpiHandleMut<'a>,
    #[signer]
    pub authority: CpiHandle<'a>,
}

#[derive(ToCpiAccounts)]
pub struct BurnChecked<'a> {
    pub from: CpiHandleMut<'a>,
    pub mint: CpiHandleMut<'a>,
    #[signer]
    pub authority: CpiHandle<'a>,
}

#[derive(ToCpiAccounts)]
pub struct Approve<'a> {
    pub to: CpiHandleMut<'a>,
    pub delegate: CpiHandle<'a>,
    #[signer]
    pub authority: CpiHandle<'a>,
}

#[derive(ToCpiAccounts)]
pub struct ApproveChecked<'a> {
    pub to: CpiHandleMut<'a>,
    pub mint: CpiHandle<'a>,
    pub delegate: CpiHandle<'a>,
    #[signer]
    pub authority: CpiHandle<'a>,
}

#[derive(ToCpiAccounts)]
pub struct Revoke<'a> {
    pub source: CpiHandleMut<'a>,
    #[signer]
    pub authority: CpiHandle<'a>,
}

#[derive(ToCpiAccounts)]
pub struct SetAuthority<'a> {
    pub account_or_mint: CpiHandleMut<'a>,
    #[signer]
    pub current_authority: CpiHandle<'a>,
}

#[derive(ToCpiAccounts)]
pub struct CloseAccount<'a> {
    pub account: CpiHandleMut<'a>,
    pub destination: CpiHandleMut<'a>,
    #[signer]
    pub authority: CpiHandle<'a>,
}

#[derive(ToCpiAccounts)]
pub struct FreezeAccount<'a> {
    pub account: CpiHandleMut<'a>,
    pub mint: CpiHandle<'a>,
    #[signer]
    pub authority: CpiHandle<'a>,
}

#[derive(ToCpiAccounts)]
pub struct ThawAccount<'a> {
    pub account: CpiHandleMut<'a>,
    pub mint: CpiHandle<'a>,
    #[signer]
    pub authority: CpiHandle<'a>,
}

#[derive(ToCpiAccounts)]
pub struct SyncNative<'a> {
    pub account: CpiHandleMut<'a>,
}

pub fn initialize_account<'a>(
    ctx: CpiContext<'a, InitializeAccount<'a>>,
) -> Result<(), ProgramError> {
    let data = token_data(1);
    ctx.invoke(&data)
}

pub fn initialize_account3<'a>(
    ctx: CpiContext<'a, InitializeAccount3<'a>>,
) -> Result<(), ProgramError> {
    let data = token_data_address(18, ctx.accounts.authority.address());
    ctx.invoke(&data)
}

pub fn initialize_mint<'a>(
    ctx: CpiContext<'a, InitializeMint<'a>>,
    decimals: u8,
    authority: &Address,
    freeze_authority: Option<&Address>,
) -> Result<(), ProgramError> {
    let mut data = token_data(0);
    data.push(decimals);
    data.extend_from_slice(authority.as_ref());
    match freeze_authority {
        Some(address) => {
            data.push(1);
            data.extend_from_slice(address.as_ref());
        }
        None => data.push(0),
    }
    ctx.invoke(&data)
}

pub fn initialize_mint2<'a>(
    ctx: CpiContext<'a, InitializeMint2<'a>>,
    decimals: u8,
    authority: &Address,
    freeze_authority: Option<&Address>,
) -> Result<(), ProgramError> {
    let mut data = token_data(20);
    data.push(decimals);
    data.extend_from_slice(authority.as_ref());
    match freeze_authority {
        Some(address) => {
            data.push(1);
            data.extend_from_slice(address.as_ref());
        }
        None => data.push(0),
    }
    ctx.invoke(&data)
}

pub fn transfer<'a>(ctx: CpiContext<'a, Transfer<'a>>, amount: u64) -> Result<(), ProgramError> {
    let data = token_data_u64(3, amount);
    ctx.invoke_with_signer_override(&data, Some((2, ctx.remaining_accounts.len() == 0)))
}

pub fn transfer_checked<'a>(
    ctx: CpiContext<'a, TransferChecked<'a>>,
    amount: u64,
    decimals: u8,
) -> Result<(), ProgramError> {
    let data = token_data_u64_u8(12, amount, decimals);
    ctx.invoke_with_signer_override(&data, Some((3, ctx.remaining_accounts.len() == 0)))
}

pub fn mint_to<'a>(ctx: CpiContext<'a, MintTo<'a>>, amount: u64) -> Result<(), ProgramError> {
    let data = token_data_u64(7, amount);
    ctx.invoke_with_signer_override(&data, Some((2, ctx.remaining_accounts.len() == 0)))
}

pub fn mint_to_checked<'a>(
    ctx: CpiContext<'a, MintToChecked<'a>>,
    amount: u64,
    decimals: u8,
) -> Result<(), ProgramError> {
    let data = token_data_u64_u8(14, amount, decimals);
    ctx.invoke_with_signer_override(&data, Some((2, ctx.remaining_accounts.len() == 0)))
}

pub fn burn<'a>(ctx: CpiContext<'a, Burn<'a>>, amount: u64) -> Result<(), ProgramError> {
    let data = token_data_u64(8, amount);
    ctx.invoke_with_signer_override(&data, Some((2, ctx.remaining_accounts.len() == 0)))
}

pub fn burn_checked<'a>(
    ctx: CpiContext<'a, BurnChecked<'a>>,
    amount: u64,
    decimals: u8,
) -> Result<(), ProgramError> {
    let data = token_data_u64_u8(15, amount, decimals);
    ctx.invoke_with_signer_override(&data, Some((2, ctx.remaining_accounts.len() == 0)))
}

pub fn approve<'a>(ctx: CpiContext<'a, Approve<'a>>, amount: u64) -> Result<(), ProgramError> {
    let data = token_data_u64(4, amount);
    ctx.invoke_with_signer_override(&data, Some((2, ctx.remaining_accounts.len() == 0)))
}

pub fn approve_checked<'a>(
    ctx: CpiContext<'a, ApproveChecked<'a>>,
    amount: u64,
    decimals: u8,
) -> Result<(), ProgramError> {
    let data = token_data_u64_u8(13, amount, decimals);
    ctx.invoke_with_signer_override(&data, Some((3, ctx.remaining_accounts.len() == 0)))
}

pub fn revoke<'a>(ctx: CpiContext<'a, Revoke<'a>>) -> Result<(), ProgramError> {
    let data = token_data(5);
    ctx.invoke_with_signer_override(&data, Some((1, ctx.remaining_accounts.len() == 0)))
}

pub fn set_authority<'a>(
    ctx: CpiContext<'a, SetAuthority<'a>>,
    authority_type: spl_token_2022::instruction::AuthorityType,
    new_authority: Option<&Address>,
) -> Result<(), ProgramError> {
    let data = token_data_set_authority(&authority_type, new_authority);
    ctx.invoke_with_signer_override(&data, Some((1, ctx.remaining_accounts.len() == 0)))
}

pub fn close_account<'a>(ctx: CpiContext<'a, CloseAccount<'a>>) -> Result<(), ProgramError> {
    let data = token_data(9);
    ctx.invoke_with_signer_override(&data, Some((2, ctx.remaining_accounts.len() == 0)))
}

pub fn freeze_account<'a>(ctx: CpiContext<'a, FreezeAccount<'a>>) -> Result<(), ProgramError> {
    let data = token_data(10);
    ctx.invoke_with_signer_override(&data, Some((2, ctx.remaining_accounts.len() == 0)))
}

pub fn thaw_account<'a>(ctx: CpiContext<'a, ThawAccount<'a>>) -> Result<(), ProgramError> {
    let data = token_data(11);
    ctx.invoke_with_signer_override(&data, Some((2, ctx.remaining_accounts.len() == 0)))
}

pub fn sync_native<'a>(ctx: CpiContext<'a, SyncNative<'a>>) -> Result<(), ProgramError> {
    let data = token_data(17);
    ctx.invoke(&data)
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        anchor_lang::{
            testing::{AccountBuffer, MIN_ACCOUNT_BUF},
        },
    };

    fn signer(address: [u8; 32]) -> AccountBuffer<{ MIN_ACCOUNT_BUF + 8 }> {
        let buffer = AccountBuffer::new();
        buffer.init(address, [9; 32], 8, true, false, false);
        buffer
    }

    #[test]
    fn remaining_signers_encode_canonical_multisig_layout() {
        let member_one = signer([4; 32]);
        let member_two = signer([5; 32]);
        let member_one_view = unsafe { member_one.view() };
        let member_two_view = unsafe { member_two.view() };
        let handles = [
            CpiHandle::readonly(&member_one_view),
            CpiHandle::readonly(&member_two_view),
        ];
        let signer_addresses = multisig_signer_addresses(&handles);

        assert_eq!(signer_addresses.len(), 2);
        assert_eq!(signer_addresses[0].as_ref(), [4; 32].as_slice());
        assert_eq!(signer_addresses[1].as_ref(), [5; 32].as_slice());
    }
}
