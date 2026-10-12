//! Minimal on-chain account metadata.
//!
//! `solana-instruction` is intentionally host-facing because its owned
//! `Instruction` representation and serialization dependencies require
//! allocation.  CPI validation only needs these three fields, so the SBF
//! build uses this layout and the host build continues to re-export the
//! canonical Solana type.

use crate::Address;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C)]
pub struct AccountMeta {
    pub pubkey: Address,
    pub is_signer: bool,
    pub is_writable: bool,
}

impl AccountMeta {
    #[inline]
    pub const fn new(pubkey: Address, is_signer: bool) -> Self {
        Self {
            pubkey,
            is_signer,
            is_writable: true,
        }
    }

    #[inline]
    pub const fn new_readonly(pubkey: Address, is_signer: bool) -> Self {
        Self {
            pubkey,
            is_signer,
            is_writable: false,
        }
    }
}
