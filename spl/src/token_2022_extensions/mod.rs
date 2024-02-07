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

pub mod confidential_transfer;
pub mod confidential_transfer_fee;
pub mod cpi_guard;
pub mod default_account_state;
pub mod group_member_pointer;
pub mod group_pointer;
pub mod immutable_owner;
pub mod interest_bearing_mint;
pub mod memo_transfer;
pub mod metadata_pointer;
pub mod mint_close_authority;
pub mod non_transferrable;
pub mod permanent_delegate;
pub mod reallocate;
pub mod token_group;
pub mod token_metadata;
pub mod transfer_fee;
pub mod transfer_hook;

pub use confidential_transfer::*;
pub use confidential_transfer_fee::*;
pub use cpi_guard::*;
pub use default_account_state::*;
pub use group_member_pointer::*;
pub use group_pointer::*;
pub use immutable_owner::*;
pub use interest_bearing_mint::*;
pub use memo_transfer::*;
pub use metadata_pointer::*;
pub use mint_close_authority::*;
pub use non_transferrable::*;
pub use permanent_delegate::*;
pub use reallocate::*;
pub use token_group::*;
pub use token_metadata::*;
pub use transfer_fee::*;
pub use transfer_hook::*;

pub use spl_token_metadata_interface;
