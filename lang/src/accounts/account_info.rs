use crate::Key;
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

impl<'info> Key for AccountInfo<'info> {
    fn key(&self) -> Pubkey {
        *self.key
    }
}
