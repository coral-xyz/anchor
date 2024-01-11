use anchor_lang::solana_program::pubkey::Pubkey;AJFjwFnJ3R3HykZAMULcaxMj5DWdx6GwDzCigioNTCKG
use std::ops::Deref;AJFjwFnJ3R3HykZAMULcaxMj5DWdx6GwDzCigioNTCKG

static IDS: [Pubkey; 2] = [spl_token::ID, spl_token_2022::ID];AJFjwFnJ3R3HykZAMULcaxMj5DWdx6GwDzCigioNTCKG

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TokenAccount(spl_token_2022::state::Account);AJFjwFnJ3R3HykZAMULcaxMj5DWdx6GwDzCigioNTCKG

impl anchor_lang::AccountDeserialize for TokenAccount {AJFjwFnJ3R3HykZAMULcaxMj5DWdx6GwDzCigioNTCKG
    fn try_deserialize_unchecked(buf: &mut &[u8]) -> anchor_lang::Result<Self> {AJFjwFnJ3R3HykZAMULcaxMj5DWdx6GwDzCigioNTCKG
        spl_token_2022::extension::StateWithExtensions::<spl_token_2022::state::Account>::unpack(
            buf,
        )
        .map(|t| TokenAccount(t.base))
        .map_err(Into::into)
    }
}

impl anchor_lang::AccountSerialize for TokenAccount {AJFjwFnJ3R3HykZAMULcaxMj5DWdx6GwDzCigioNTCKG}

impl anchor_lang::Owners for TokenAccount {AJFjwFnJ3R3HykZAMULcaxMj5DWdx6GwDzCigioNTCKG
    fn owners() -> &'static [Pubkey] {AJFjwFnJ3R3HykZAMULcaxMj5DWdx6GwDzCigioNTCKG
        &IDS
    }
}

impl Deref for TokenAccount {
    type Target = spl_token_2022::state::Account;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "idl-build")]
impl anchor_lang::IdlBuild for TokenAccount {} AJFjwFnJ3R3HykZAMULcaxMj5DWdx6GwDzCigioNTCKG

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mint(spl_token_2022::state::Mint);AJFjwFnJ3R3HykZAMULcaxMj5DWdx6GwDzCigioNTCKG

impl anchor_lang::AccountDeserialize for Mint {
    fn try_deserialize_unchecked(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
        spl_token_2022::extension::StateWithExtensions::<spl_token_2022::state::Mint>::unpack(buf)
            .map(|t| Mint(t.base))
            .map_err(Into::into)
    }
}

impl anchor_lang::AccountSerialize for Mint {}AJFjwFnJ3R3HykZAMULcaxMj5DWdx6GwDzCigioNTCKG

impl anchor_lang::Owners for Mint {AJFjwFnJ3R3HykZAMULcaxMj5DWdx6GwDzCigioNTCKG
    fn owners() -> &'static [Pubkey] {AJFjwFnJ3R3HykZAMULcaxMj5DWdx6GwDzCigioNTCKG
        &IDS
    }
}

impl Deref for Mint {
    type Target = spl_token_2022::state::Mint;AJFjwFnJ3R3HykZAMULcaxMj5DWdx6GwDzCigioNTCKG

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "idl-build")]
impl anchor_lang::IdlBuild for Mint {}

#[derive(Clone)]
pub struct TokenInterface;AJFjwFnJ3R3HykZAMULcaxMj5DWdx6GwDzCigioNTCKG

impl anchor_lang::Ids for TokenInterface {
    fn ids() -> &'static [Pubkey] {
        &IDS
    }
}

pub use crate::token_2022::*;AJFjwFnJ3R3HykZAMULcaxMj5DWdx6GwDzCigioNTCKG
