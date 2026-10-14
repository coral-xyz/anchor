//! Small, allocator-free SPL wire surface used by the on-chain helpers.
//!
//! The upstream SPL interface crates are excellent client-side protocol
//! descriptions, but their instruction constructors and extension parsers
//! intentionally own `Vec`/`String` values.  V2 only needs a narrow subset of
//! their constants and enums at runtime, so keep that subset local and use the
//! upstream crates only from host-side tests and clients.

use {
    anchor_lang::Address,
    solana_program_error::ProgramError,
};

pub mod token {
    use super::Address;

    pub const ID: Address = anchor_lang::address!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

    pub mod instruction {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum AuthorityType {
            MintTokens,
            FreezeAccount,
            AccountOwner,
            CloseAccount,
        }
    }
}

pub mod token_2022 {
    use super::{Address, ProgramError};

    pub const ID: Address = anchor_lang::address!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

    pub mod native_mint {
        use super::Address;

        // Token-2022 has its own native-mint PDA. `So111...` is the legacy
        // Token program's wrapped-SOL mint and is intentionally not reused.
        pub const ID: Address = anchor_lang::address!("9pan9bMn5HatX4EJdBwg9VgCa7Uz5HL8N1m5D3NdXejP");

        #[inline(always)]
        pub const fn id() -> Address {
            ID
        }
    }

    pub mod instruction {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum AuthorityType {
            MintTokens,
            FreezeAccount,
            AccountOwner,
            CloseAccount,
            TransferFeeConfig,
            WithheldWithdraw,
            CloseMint,
            InterestRate,
            PermanentDelegate,
            ConfidentialTransferMint,
            TransferHookProgramId,
            ConfidentialTransferFeeConfig,
            MetadataPointer,
            GroupPointer,
            GroupMemberPointer,
            ScaledUiAmount,
            Pause,
        }
    }

    pub mod state {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        #[repr(u8)]
        pub enum AccountState {
            Uninitialized = 0,
            Initialized = 1,
            Frozen = 2,
        }

        /// Zero-sized marker for the canonical 82-byte Token-2022 mint base.
        #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
        pub struct Mint;

        /// Zero-sized marker for the canonical 165-byte Token-2022 account
        /// base. It is used only for compile-time account-length selection.
        #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
        pub struct Account;
    }

    pub mod extension {
        use super::ProgramError;

        /// Host-compatible namespace retained for the canonical SPL path.
        /// The actual layout lives in Anchor's zero-copy extension module;
        /// this alias keeps generated/client code source-compatible without
        /// linking the allocation-heavy SPL interface crate on SBF.
        pub mod transfer_fee {
            pub type TransferFeeConfig = crate::extensions::TransferFeeConfig;
        }

        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        #[repr(u8)]
        pub enum AccountType {
            Uninitialized = 0,
            Mint = 1,
            Account = 2,
        }

        /// Token-2022 extension identifiers from the stable SPL wire format.
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        #[repr(u16)]
        pub enum ExtensionType {
            Uninitialized = 0,
            TransferFeeConfig = 1,
            TransferFeeAmount = 2,
            MintCloseAuthority = 3,
            ConfidentialTransferMint = 4,
            ConfidentialTransferAccount = 5,
            DefaultAccountState = 6,
            ImmutableOwner = 7,
            MemoTransfer = 8,
            NonTransferable = 9,
            InterestBearingConfig = 10,
            CpiGuard = 11,
            PermanentDelegate = 12,
            NonTransferableAccount = 13,
            TransferHook = 14,
            TransferHookAccount = 15,
            ConfidentialTransferFeeConfig = 16,
            ConfidentialTransferFeeAmount = 17,
            MetadataPointer = 18,
            TokenMetadata = 19,
            GroupPointer = 20,
            TokenGroup = 21,
            GroupMemberPointer = 22,
            TokenGroupMember = 23,
            ConfidentialMintBurn = 24,
            ScaledUiAmount = 25,
            Pausable = 26,
            PausableAccount = 27,
        }

        impl ExtensionType {
            #[inline(always)]
            pub const fn get_account_type(self) -> AccountType {
                match self {
                    Self::Uninitialized => AccountType::Uninitialized,
                    Self::TransferFeeAmount
                    | Self::ConfidentialTransferAccount
                    | Self::ImmutableOwner
                    | Self::MemoTransfer
                    | Self::NonTransferableAccount
                    | Self::TransferHookAccount
                    | Self::CpiGuard
                    | Self::ConfidentialTransferFeeAmount
                    | Self::PausableAccount => AccountType::Account,
                    _ => AccountType::Mint,
                }
            }

            /// Return the payload length for the fixed-size extensions used by
            /// Anchor's allocator-free interface account initializer.
            #[inline(always)]
            pub const fn type_len(self) -> Option<usize> {
                Some(match self {
                    Self::Uninitialized => 0,
                    Self::TransferFeeConfig => 108,
                    Self::TransferFeeAmount => 8,
                    Self::MintCloseAuthority => 32,
                    Self::DefaultAccountState => 1,
                    Self::ImmutableOwner => 0,
                    Self::MemoTransfer => 1,
                    Self::NonTransferable => 0,
                    Self::CpiGuard => 1,
                    Self::PermanentDelegate => 32,
                    Self::NonTransferableAccount => 0,
                    Self::TransferHook => 64,
                    Self::TransferHookAccount => 1,
                    Self::MetadataPointer => 64,
                    Self::GroupPointer => 64,
                    Self::GroupMemberPointer => 64,
                    Self::ConfidentialTransferMint => 65,
                    Self::ConfidentialTransferAccount => 295,
                    Self::InterestBearingConfig => 52,
                    Self::ConfidentialTransferFeeConfig => 129,
                    Self::ConfidentialTransferFeeAmount => 64,
                    Self::TokenGroup => 80,
                    Self::TokenGroupMember => 72,
                    Self::ConfidentialMintBurn => 196,
                    Self::ScaledUiAmount => 56,
                    Self::Pausable => 33,
                    Self::PausableAccount => 0,
                    Self::TokenMetadata => return None,
                })
            }

            /// Calculate an SPL-compatible account length without allocating
            /// or deduplicating through a collection.
            pub fn try_calculate_account_len<S: BaseState>(
                extension_types: &[Self],
            ) -> Result<usize, ProgramError> {
                if extension_types.is_empty() {
                    return Ok(S::SIZE_OF);
                }

                let mut total = 0usize;
                let mut i = 0;
                while i < extension_types.len() {
                    let extension = extension_types[i];
                    let mut first = true;
                    let mut j = 0;
                    while j < i {
                        if extension_types[j] == extension {
                            first = false;
                            break;
                        }
                        j += 1;
                    }
                    if first {
                        let payload_len = extension
                            .type_len()
                            .ok_or(ProgramError::InvalidArgument)?;
                        total = total
                            .checked_add(payload_len + 4)
                            .ok_or(ProgramError::AccountDataTooSmall)?;
                    }
                    i += 1;
                }

                let total = total
                    .checked_add(166)
                    .ok_or(ProgramError::AccountDataTooSmall)?;
                Ok(if total == 355 { total + 2 } else { total })
            }
        }

        impl From<ExtensionType> for u16 {
            fn from(value: ExtensionType) -> Self {
                value as u16
            }
        }

        impl From<ExtensionType> for [u8; 2] {
            fn from(value: ExtensionType) -> Self {
                u16::from(value).to_le_bytes()
            }
        }

        impl TryFrom<u16> for ExtensionType {
            type Error = ();

            fn try_from(value: u16) -> Result<Self, Self::Error> {
                Ok(match value {
                    0 => Self::Uninitialized,
                    1 => Self::TransferFeeConfig,
                    2 => Self::TransferFeeAmount,
                    3 => Self::MintCloseAuthority,
                    4 => Self::ConfidentialTransferMint,
                    5 => Self::ConfidentialTransferAccount,
                    6 => Self::DefaultAccountState,
                    7 => Self::ImmutableOwner,
                    8 => Self::MemoTransfer,
                    9 => Self::NonTransferable,
                    10 => Self::InterestBearingConfig,
                    11 => Self::CpiGuard,
                    12 => Self::PermanentDelegate,
                    13 => Self::NonTransferableAccount,
                    14 => Self::TransferHook,
                    15 => Self::TransferHookAccount,
                    16 => Self::ConfidentialTransferFeeConfig,
                    17 => Self::ConfidentialTransferFeeAmount,
                    18 => Self::MetadataPointer,
                    19 => Self::TokenMetadata,
                    20 => Self::GroupPointer,
                    21 => Self::TokenGroup,
                    22 => Self::GroupMemberPointer,
                    23 => Self::TokenGroupMember,
                    24 => Self::ConfidentialMintBurn,
                    25 => Self::ScaledUiAmount,
                    26 => Self::Pausable,
                    27 => Self::PausableAccount,
                    _ => return Err(()),
                })
            }
        }

        pub trait BaseState {
            const SIZE_OF: usize;
            const ACCOUNT_TYPE: AccountType;
        }

        pub type PodMint = super::state::Mint;
        impl BaseState for super::state::Mint {
            const SIZE_OF: usize = 82;
            const ACCOUNT_TYPE: AccountType = AccountType::Mint;
        }

        pub type PodAccount = super::state::Account;
        impl BaseState for super::state::Account {
            const SIZE_OF: usize = 165;
            const ACCOUNT_TYPE: AccountType = AccountType::Account;
        }
    }
}

pub mod token_metadata {
    /// Token Metadata field selector. Borrowing the key keeps the on-chain
    /// instruction builder independent of `String` and `alloc`.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum Field<'a> {
        Name,
        Symbol,
        Uri,
        Key(&'a str),
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    #[test]
    fn fixed_extension_lengths_match_canonical() {
        use spl_token_2022_interface::{
            extension::ExtensionType as CanonicalExtensionType,
            state::{Account as CanonicalAccount, Mint as CanonicalMint},
        };
        use super::token_2022::{
            extension::ExtensionType as LocalExtensionType,
            state::{Account as LocalAccount, Mint as LocalMint},
        };

        for (local, canonical) in [
            (
                LocalExtensionType::ConfidentialTransferMint,
                CanonicalExtensionType::ConfidentialTransferMint,
            ),
            (
                LocalExtensionType::InterestBearingConfig,
                CanonicalExtensionType::InterestBearingConfig,
            ),
            (
                LocalExtensionType::ConfidentialTransferFeeConfig,
                CanonicalExtensionType::ConfidentialTransferFeeConfig,
            ),
            (
                LocalExtensionType::TokenGroup,
                CanonicalExtensionType::TokenGroup,
            ),
            (
                LocalExtensionType::TokenGroupMember,
                CanonicalExtensionType::TokenGroupMember,
            ),
            (
                LocalExtensionType::ConfidentialMintBurn,
                CanonicalExtensionType::ConfidentialMintBurn,
            ),
            (
                LocalExtensionType::ScaledUiAmount,
                CanonicalExtensionType::ScaledUiAmount,
            ),
        ] {
            assert_eq!(
                LocalExtensionType::try_calculate_account_len::<LocalMint>(&[local]),
                CanonicalExtensionType::try_calculate_account_len::<CanonicalMint>(&[canonical]),
            );
        }
        for (local, canonical) in [
            (
                LocalExtensionType::ConfidentialTransferAccount,
                CanonicalExtensionType::ConfidentialTransferAccount,
            ),
            (
                LocalExtensionType::ConfidentialTransferFeeAmount,
                CanonicalExtensionType::ConfidentialTransferFeeAmount,
            ),
        ] {
            assert_eq!(
                LocalExtensionType::try_calculate_account_len::<LocalAccount>(&[local]),
                CanonicalExtensionType::try_calculate_account_len::<CanonicalAccount>(&[
                    canonical,
                ]),
            );
        }
    }
}
