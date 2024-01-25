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

pub mod group_member_pointer;
pub mod group_pointer;
pub mod metadata_pointer;
pub mod token_group;
pub mod token_metadata;

pub use group_member_pointer::*;
pub use group_pointer::*;
pub use metadata_pointer::*;
pub use token_group::*;
pub use token_metadata::*;

pub use spl_token_metadata_interface;