use crate::{env, Host, Program};
use solana_program::account_info::next_account_info;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction::{SystemError, SystemInstruction};
use solana_program::system_program;
use std::fmt::Debug;

pub use system_program::ID;

// System program emulation to plug into the fuzzing host. Not all instructions
// are implemented, but PRs are welcome.
#[derive(Debug)]
pub struct SystemProgram;

impl Program for SystemProgram {
    fn entry(
        &self,
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        ix_data: &[u8],
    ) -> ProgramResult {
        let ix: SystemInstruction =
            bincode::deserialize(ix_data).map_err(|_| ProgramError::InvalidInstructionData)?;

        match ix {
            SystemInstruction::CreateAccount {
                lamports,
                space,
                owner,
            } => self.create_account(accounts, lamports, space, owner),
            SystemInstruction::Transfer { lamports } => self.transfer(accounts, lamports),
            SystemInstruction::CreateAccountWithSeed {
                base,
                seed,
                lamports,
                space,
                owner,
            } => self.create_account_with_seed(accounts, base, seed, lamports, space, owner),
            SystemInstruction::Assign { owner } => self.assign(accounts, owner),
            SystemInstruction::AdvanceNonceAccount => self.advance_nonce_account(accounts),
            SystemInstruction::WithdrawNonceAccount(lamports) => {
                self.withdraw_nonce_account(accounts, lamports)
            }
            SystemInstruction::InitializeNonceAccount(entity) => {
                self.initialize_nonce_account(accounts, entity)
            }
            SystemInstruction::AuthorizeNonceAccount(entity) => {
                self.authorize_nonce_account(accounts, entity)
            }
            SystemInstruction::Allocate { space } => self.allocate(accounts, space),
            SystemInstruction::AllocateWithSeed {
                base,
                seed,
                space,
                owner,
            } => self.allocate_with_seed(accounts, base, seed, space, owner),
            SystemInstruction::AssignWithSeed { base, seed, owner } => {
                self.assign_with_seed(accounts, base, seed, owner)
            }
            SystemInstruction::TransferWithSeed {
                lamports,
                from_seed,
                from_owner,
            } => self.transfer_with_seed(accounts, lamports, from_seed, from_owner),
        }
    }

    fn id(&self) -> Pubkey {
        system_program::ID
    }
}

impl SystemProgram {
    fn create_account<'info>(
        &self,
        accounts: &[AccountInfo<'info>],
        lamports: u64,
        space: u64,
        owner: Pubkey,
    ) -> ProgramResult {
        let acc_infos = &mut accounts.into_iter();

        let from = next_account_info(acc_infos)?;
        let created = next_account_info(acc_infos)?;

        if !from.is_signer {
            panic!("From not signer");
        }
        if !created.is_signer {
            panic!("Created not signer");
        }
        if !from.is_writable {
            panic!("From not writable");
        }
        if !created.is_writable {
            panic!("Created not writable");
        }

        if **created.lamports.borrow() > 0 {
            panic!("{}", SystemError::AccountAlreadyInUse);
        }

        // Safe because access to this method is single threaded.
        created.data.replace(
            env()
                .accounts
                .storage()
                .alloc_slice_fill_copy(space as usize, 0u8),
        );

        let owner_og_ptr_const = created.owner as *const Pubkey;
        let owner_og_ptr_mut = owner_og_ptr_const as *mut Pubkey;

        unsafe {
            std::ptr::write_unaligned(owner_og_ptr_mut, owner);
        }

        **from.lamports.borrow_mut() -= lamports;
        **created.lamports.borrow_mut() += lamports;

        Ok(())
    }

    fn transfer(&self, accounts: &[AccountInfo], lamports: u64) -> ProgramResult {
        let acc_infos = &mut accounts.into_iter();

        let from = next_account_info(acc_infos)?;
        let to = next_account_info(acc_infos)?;

        if !from.is_signer {
            panic!("From not signer");
        }
        if !from.is_writable {
            panic!("From not writable");
        }
        if !to.is_writable {
            panic!("To not writable");
        }

        **from.lamports.borrow_mut() -= lamports;
        **to.lamports.borrow_mut() += lamports;

        Ok(())
    }

    fn create_account_with_seed(
        &self,
        accounts: &[AccountInfo],
        base: Pubkey,
        seed: String,
        lamports: u64,
        space: u64,
        owner: Pubkey,
    ) -> ProgramResult {
        unimplemented!()
    }

    fn assign(&self, _accounts: &[AccountInfo], owner: Pubkey) -> ProgramResult {
        unimplemented!()
    }

    fn advance_nonce_account(&self, _accounts: &[AccountInfo]) -> ProgramResult {
        unimplemented!()
    }

    fn withdraw_nonce_account(&self, _accounts: &[AccountInfo], _lamports: u64) -> ProgramResult {
        unimplemented!()
    }

    fn initialize_nonce_account(
        &self,
        _accounts: &[AccountInfo],
        _entity: Pubkey,
    ) -> ProgramResult {
        unimplemented!()
    }

    fn authorize_nonce_account(&self, _accounts: &[AccountInfo], _entity: Pubkey) -> ProgramResult {
        unimplemented!()
    }

    fn allocate(&self, _accounts: &[AccountInfo], _space: u64) -> ProgramResult {
        unimplemented!()
    }

    fn allocate_with_seed(
        &self,
        _accounts: &[AccountInfo],
        _base: Pubkey,
        _seed: String,
        _space: u64,
        _owner: Pubkey,
    ) -> ProgramResult {
        unimplemented!()
    }

    fn assign_with_seed(
        &self,
        _accounts: &[AccountInfo],
        _base: Pubkey,
        _seed: String,
        _owner: Pubkey,
    ) -> ProgramResult {
        unimplemented!()
    }

    fn transfer_with_seed(
        &self,
        _accounts: &[AccountInfo],
        _lamports: u64,
        _from_seed: String,
        _from_owner: Pubkey,
    ) -> ProgramResult {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::instruction::{AccountMeta, Instruction};

    #[test]
    fn system_program_creates_account() {
        let host = Host::new();

        let owner = Pubkey::new_unique();
        let mut from = host.accounts().new_sol_account(1000);
        from.is_writable = true;
        from.is_signer = true;

        let mut created = host.accounts().new_sol_account(0);
        created.is_writable = true;
        created.is_signer = true;

        let ix = {
            let data = bincode::serialize(&SystemInstruction::CreateAccount {
                lamports: 999,
                space: 1234,
                owner,
            })
            .unwrap();

            Instruction {
                program_id: system_program::ID,
                data,
                accounts: vec![
                    AccountMeta::new(*from.key, true),
                    AccountMeta::new(*created.key, true),
                ],
            }
        };

        let accounts = &[from.clone(), created.clone()];

        assert_eq!(*created.data.borrow_mut(), &[]);
        assert_eq!(**from.lamports.borrow(), 1000);
        assert_eq!(*created.owner, Pubkey::new_from_array([0u8; 32]));
        host.invoke(&ix, accounts, &[]).unwrap();
        assert_eq!(*created.data.borrow_mut(), &[0u8; 1234]);
        assert_eq!(**created.lamports.borrow(), 999);
        assert_eq!(**from.lamports.borrow(), 1);
        assert_eq!(*created.owner, owner);
    }

    #[test]
    fn system_program_transfer() {
        let host = Host::new();

        let mut from = host.accounts().new_sol_account(1000);
        from.is_writable = true;
        from.is_signer = true;

        let mut to = host.accounts().new_sol_account(0);
        to.is_writable = true;

        let ix = {
            let data = bincode::serialize(&SystemInstruction::Transfer { lamports: 999 }).unwrap();

            Instruction {
                program_id: system_program::ID,
                data,
                accounts: vec![
                    AccountMeta::new(*from.key, true),
                    AccountMeta::new(*to.key, true),
                ],
            }
        };

        let accounts = &[from.clone(), to.clone()];

        assert_eq!(**from.lamports.borrow(), 1000);
        assert_eq!(**to.lamports.borrow(), 0);
        host.invoke(&ix, accounts, &[]).unwrap();
        assert_eq!(**from.lamports.borrow(), 1);
        assert_eq!(**to.lamports.borrow(), 999);
    }
}
