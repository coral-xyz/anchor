use std::marker::PhantomData;

use anchor_lang::prelude::{borsh::*, *};

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub struct QuickCheck<T> {
    #[borsh_skip]
    _marker: PhantomData<T>,
}

impl<T> AccountSerialize for QuickCheck<T> {}

impl<T> AccountDeserialize for QuickCheck<T> {
    fn try_deserialize_unchecked(_buf: &mut &[u8]) -> Result<Self> {
        Ok(QuickCheck {
            _marker: PhantomData,
        })
    }
}

impl<T: Owner> Owner for QuickCheck<T> {
    fn owner() -> Pubkey {
        T::owner()
    }
}
