use crate::Accounts;
use solana_sdk::account_info::AccountInfo;
use solana_sdk::pubkey::Pubkey;

/// Provides non-argument inputs to the program.
pub struct Context<'a, 'b, 'c, 'info, T> {
    /// Currently executing program id.
    pub program_id: &'a Pubkey,
    /// Deserialized accounts.
    pub accounts: &'b mut T,
    /// Remaining accounts given but not deserialized or validated.
    pub remaining_accounts: &'c [AccountInfo<'info>],
}

impl<'a, 'b, 'c, 'info, T> Context<'a, 'b, 'c, 'info, T> {
    pub fn new(
        program_id: &'a Pubkey,
        accounts: &'b mut T,
        remaining_accounts: &'c [AccountInfo<'info>],
    ) -> Self {
        Self {
            accounts,
            program_id,
            remaining_accounts,
        }
    }
}

/// Context speciying non-argument inputs for cross-program-invocations.
pub struct CpiContext<'a, 'b, 'c, 'info, T: Accounts<'info>> {
    pub accounts: T,
    pub program: AccountInfo<'info>,
    pub signer_seeds: &'a [&'b [&'c [u8]]],
}

impl<'a, 'b, 'c, 'info, T: Accounts<'info>> CpiContext<'a, 'b, 'c, 'info, T> {
    pub fn new(program: AccountInfo<'info>, accounts: T) -> Self {
        Self {
            accounts,
            program,
            signer_seeds: &[],
        }
    }

    pub fn new_with_signer(
        accounts: T,
        program: AccountInfo<'info>,
        signer_seeds: &'a [&'b [&'c [u8]]],
    ) -> Self {
        Self {
            accounts,
            program,
            signer_seeds,
        }
    }
}
