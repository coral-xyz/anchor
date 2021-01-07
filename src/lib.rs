use solana_sdk::account_info::AccountInfo;
use solana_sdk::program_error::ProgramError;
use solana_sdk::pubkey::Pubkey;
use std::io::Write;
use std::ops::{Deref, DerefMut};

pub use anchor_attribute_access_control::access_control;
pub use anchor_attribute_account::account;
pub use anchor_attribute_program::program;
pub use anchor_derive_accounts::Accounts;
pub use borsh::{BorshDeserialize as AnchorDeserialize, BorshSerialize as AnchorSerialize};

pub trait Accounts<'info>: Sized {
    fn try_accounts(program_id: &Pubkey, from: &[AccountInfo<'info>])
        -> Result<Self, ProgramError>;
}

pub trait AccountSerialize {
    fn try_serialize<W: Write>(&self, writer: &mut W) -> Result<(), ProgramError>;
}

pub trait AccountDeserialize: Sized {
    fn try_deserialize(buf: &mut &[u8]) -> Result<Self, ProgramError>;
}

pub struct ProgramAccount<'a, T: AccountSerialize + AccountDeserialize> {
    pub info: AccountInfo<'a>,
    pub account: T,
}

impl<'a, T: AccountSerialize + AccountDeserialize> ProgramAccount<'a, T> {
    pub fn new(info: AccountInfo<'a>, account: T) -> ProgramAccount<'a, T> {
        Self { info, account }
    }

    pub fn try_from(info: &AccountInfo<'a>) -> Result<ProgramAccount<'a, T>, ProgramError> {
        let mut data: &[u8] = &info.try_borrow_data()?;
        Ok(ProgramAccount::new(
            info.clone(),
            T::try_deserialize(&mut data)?,
        ))
    }
}

impl<'a, T: AccountSerialize + AccountDeserialize> Deref for ProgramAccount<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.account
    }
}

impl<'a, T: AccountSerialize + AccountDeserialize> DerefMut for ProgramAccount<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.account
    }
}

pub struct Context<'a, 'b, T> {
    pub accounts: &'a mut T,
    pub program_id: &'b Pubkey,
}

pub mod prelude {
    pub use super::{
        access_control, account, program, AccountDeserialize, AccountSerialize, Accounts,
        AnchorDeserialize, AnchorSerialize, Context, ProgramAccount,
    };

    pub use solana_program::msg;
    pub use solana_sdk::account_info::next_account_info;
    pub use solana_sdk::account_info::AccountInfo;
    pub use solana_sdk::entrypoint::ProgramResult;
    pub use solana_sdk::program_error::ProgramError;
    pub use solana_sdk::pubkey::Pubkey;
}
