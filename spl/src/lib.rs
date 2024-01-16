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
