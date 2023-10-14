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

#[cfg(test)]
mod tests {
    #![allow(unused_must_use)]
    use std::{cell::RefCell, rc::Rc};

    use solana_program::pubkey;

    use super::*;

    // Let's fake spl_token for our tests
    mod spl_token {
        use super::*;

        pub const ID: Pubkey = pubkey!("ucy2UcT2KPb99LqhzSSsNN6WFfvHydLZta9Mfz9DLkU");
        pub mod state {
            #[derive(Clone, Copy, Debug, Default, PartialEq)]
            pub struct Mint; // just fake it
        }
    }

    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct Mint(spl_token::state::Mint);

    impl anchor_lang::AccountDeserialize for Mint {
        fn try_deserialize_unchecked(_buf: &mut &[u8]) -> anchor_lang::Result<Self> {
            Ok(Mint(spl_token::state::Mint))
        }
    }

    impl anchor_lang::AccountSerialize for Mint {}

    impl anchor_lang::Owner for Mint {
        fn owner() -> Pubkey {
            spl_token::ID
        }
    }

    // #[program]
    // mod candy_machine { .. }
    mod program {
        use super::*;

        #[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
        pub struct CandyMachine;

        pub const ID: Pubkey = pubkey!("D4fb9FSb7645bNZxSQ1P6abvpmHF5Qm7BtHnoN9jvbX6");

        impl Owner for CandyMachine {
            fn owner() -> Pubkey {
                ID // actually users would write crate::ID
            }
        }
    }

    const ACCOUNT_KEY: Pubkey = pubkey!("7Qn61qamvGdiLZjmXBr3UHL5FhcSQdF1ieQvVgKBNWLQ");

    fn new_account_info((data, lamport, owner): &mut ([u8; 1024], u64, Pubkey)) -> AccountInfo<'_> {
        AccountInfo {
            key: &ACCOUNT_KEY,
            is_signer: false,
            is_writable: true,
            lamports: Rc::new(RefCell::new(lamport)),
            data: Rc::new(RefCell::new(data)),
            owner,
            executable: false,
            rent_epoch: 0,
        }
    }
    fn deserialized_value<T>() -> QuickCheck<T> {
        QuickCheck {
            _marker: PhantomData,
        }
    }

    #[test]
    fn test_quick_check_good_case() {
        let mut good_mint_data = ([0; 1024], 10959, Mint::owner());
        let good_mint_info = new_account_info(&mut good_mint_data);
        let good_mint = Account::<QuickCheck<Mint>>::try_from(&good_mint_info);
        assert!(good_mint.is_ok());
        assert_eq!(*good_mint.unwrap(), deserialized_value());
    }

    #[test]
    fn test_quick_check_bad_case() {
        let mut bad_mint_data = (
            [0; 1024],
            10959,
            pubkey!("9oCD1F3CamZLFbra4TM8sh7SPFTCTNP8HQ6evBwY583m"),
        );
        let bad_mint_info = new_account_info(&mut bad_mint_data);
        let bad_mint = Account::<QuickCheck<Mint>>::try_from(&bad_mint_info);
        assert!(bad_mint.is_err());
        assert_eq!(
            bad_mint.err(),
            Some(Error::from(ErrorCode::AccountOwnedByWrongProgram))
        );
    }

    #[test]
    fn test_quick_check_using_program_type() {
        let mut candy_data = ([0; 1024], 10959, program::ID);
        let candy_info = new_account_info(&mut candy_data);
        let candy = Account::<QuickCheck<program::CandyMachine>>::try_from(&candy_info);
        assert!(candy.is_ok());
        assert_eq!(*candy.unwrap(), deserialized_value());
    }
}
