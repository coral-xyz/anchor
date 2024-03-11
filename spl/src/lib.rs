#![cfg_attr(docsrs, feature(doc_auto_cfg))]

//! Anchor CPI wrappers for popular programs in the Solana ecosystem.

#[cfg(feature = "associated_token")]
pub mod associated_token;

#[cfg(feature = "mint")]
pub mod mint;

#[cfg(feature = "token")]
pub mod token;

#[cfg(feature = "token_2022")]
pub mod token_2022;

#[cfg(feature = "token_2022_extensions")]
pub mod token_2022_extensions;

#[cfg(feature = "token_2022")]
pub mod token_interface;

#[cfg(feature = "dex")]
pub mod dex;

#[cfg(feature = "governance")]
pub mod governance;

#[cfg(feature = "stake")]
pub mod stake;

#[cfg(feature = "metadata")]
pub mod metadata;

#[cfg(feature = "memo")]
pub mod memo;

use anchor_lang::Result;
use solana_program::program_pack::Pack;
use spl_token_2022::{extension::ExtensionType, state::Mint};

pub type ExtensionsVec = Vec<ExtensionType>;

pub fn find_mint_account_size(extensions: Option<&ExtensionsVec>) -> Result<usize> {
    if let Some(extensions) = extensions {
        Ok(ExtensionType::try_calculate_account_len::<Mint>(
            extensions,
        )?)
    } else {
        Ok(Mint::LEN)
    }
}
