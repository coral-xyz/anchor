use anchor_lang::error::ErrorCode;
use std::ops::Deref;

pub use mpl_core;
pub use mpl_core::ID;

#[derive(Clone, Debug, PartialEq)]
pub struct AssetAccount(mpl_core::Asset);

impl anchor_lang::AccountDeserialize for AssetAccount {
    fn try_deserialize(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
        let asset = Self::try_deserialize_unchecked(buf)?;
        if asset.base.key == mpl_core::types::Key::Uninitialized {
            return Err(ErrorCode::AccountNotInitialized.into());
        }

        Ok(asset)
    }

    fn try_deserialize_unchecked(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
        let asset = mpl_core::Asset::deserialize(buf)?;
        Ok(Self(asset))
    }
}

impl anchor_lang::AccountSerialize for AssetAccount {}

impl anchor_lang::Owner for AssetAccount {
    fn owner() -> Pubkey {
        ID
    }
}

impl Deref for AssetAccount {
    type Target = mpl_core::Asset;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BaseAssetV1Account(mpl_core::accounts::BaseAssetV1);

impl anchor_lang::AccountDeserialize for BaseAssetV1Account {
    fn try_deserialize(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
        let base_asset = Self::try_deserialize_unchecked(buf)?;
        if base_asset.key == mpl_core::types::Key::Uninitialized {
            return Err(ErrorCode::AccountNotInitialized.into());
        }

        Ok(base_asset)
    }

    fn try_deserialize_unchecked(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
        let base_asset = mpl_core::accounts::BaseAssetV1::deserialize(buf)?;
        Ok(Self(base_asset))
    }
}

impl anchor_lang::AccountSerialize for BaseAssetV1Account {}

impl anchor_lang::Owner for BaseAssetV1Account {
    fn owner() -> Pubkey {
        ID
    }
}

impl Deref for BaseAssetV1Account {
    type Target = mpl_core::accounts::BaseAssetV1;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
pub struct MplCore;

impl anchor_lang::Id for MplCore {
    fn id() -> Pubkey {
        mpl_core::ID
    }
}
