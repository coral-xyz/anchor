//! The fuzz modules provides utilities to facilitate fuzzing anchor programs.

use bumpalo::Bump;
use num_derive::ToPrimitive;
use safe_transmute::to_bytes::transmute_to_bytes;
use solana_program::account_info::next_account_info;
use solana_program::account_info::AccountInfo;
use solana_program::bpf_loader;
use solana_program::clock::Epoch;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::system_instruction::{SystemError, SystemInstruction};
use solana_program::system_program;
use solana_program::sysvar::{self, Sysvar};
use spl_token::state::{Account as TokenAccount, Mint};
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::mem::size_of;
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard};
use thiserror::Error;

lazy_static::lazy_static! {
    static ref ENV: Arc<Environment> = Arc::new(Environment::new());
}

// Global host environment.
pub fn env<'info>() -> Arc<Environment> {
    ENV.clone()
}

// The host execution environment.
#[derive(Debug)]
pub struct Environment {
    // All registered programs that can be invoked.
    programs: HashMap<Pubkey, Box<dyn Program>>,
    // The currently executing program.
    current_program: RefCell<Option<Pubkey>>,
    // Account storage.
    accounts: AccountStore,
}

impl Environment {
    pub fn new() -> Environment {
        let mut env = Environment {
            programs: HashMap::new(),
            current_program: RefCell::new(None),
            accounts: AccountStore::new(),
        };
        env.register(Box::new(SystemProgram));
        env.register(Box::new(SplToken));
        env
    }

    pub fn accounts(&self) -> &AccountStore {
        &self.accounts
    }

    // Registers the program on the environment so that it can be invoked via
    // CPI.
    pub fn register(&mut self, program: Box<dyn Program>) {
        self.programs.insert(program.id(), program);
    }

    // Performs a cross program invocation.
    pub fn invoke(
        &self,
        ix: &Instruction,
        accounts: &[AccountInfo],
        seeds: &[&[&[u8]]],
    ) -> ProgramResult {
        // If seeds were given, then calculate the expected PDA.
        let pda = {
            match *self.current_program.borrow() {
                None => None,
                Some(current_program) => match seeds.len() > 0 {
                    false => None,
                    true => {
                        Some(Pubkey::create_program_address(seeds[0], &current_program).unwrap())
                    }
                },
            }
        };

        // Set the current program.
        self.current_program.replace(Some(ix.program_id));

        // Invoke the current program.
        let program = self.programs.get(&ix.program_id).unwrap();
        let account_infos: Vec<AccountInfo> = ix
            .accounts
            .iter()
            .map(|meta| {
                let mut acc_info = accounts
                    .iter()
                    .find(|info| *info.key == meta.pubkey)
                    .unwrap()
                    .clone();
                // If a PDA was given, market it as signer.
                if let Some(pda) = pda {
                    if acc_info.key == &pda {
                        acc_info.is_signer = true;
                    }
                }
                acc_info
            })
            .collect();
        program.entry(&ix.program_id, &account_infos, &ix.data)
    }
}

// Not acutally Sync. Implemented so that we can use the Environment as a
// lazy static without using locks (which is inconvenient and can cause
// deadlock). The Environment, as presently constructed, should never be
// used across threads.
unsafe impl std::marker::Sync for Environment {}

#[derive(Debug)]
pub struct AccountStore {
    // Storage bytes.
    storage: Bump,
}

impl AccountStore {
    pub fn new() -> Self {
        Self {
            storage: Bump::new(),
        }
    }

    pub fn storage(&self) -> &Bump {
        &self.storage
    }

    pub fn new_sol_account(&self, lamports: u64) -> AccountInfo {
        AccountInfo::new(
            random_pubkey(&self.storage),
            true,
            false,
            self.storage.alloc(lamports),
            &mut [],
            &system_program::ID,
            false,
            Epoch::default(),
        )
    }

    pub fn new_token_mint(&self) -> AccountInfo {
        let rent = Rent::default();
        let data = self.storage.alloc_slice_fill_copy(Mint::LEN, 0u8);
        let mut mint = Mint::default();
        mint.is_initialized = true;
        Mint::pack(mint, data).unwrap();
        AccountInfo::new(
            random_pubkey(&self.storage),
            false,
            true,
            self.storage.alloc(rent.minimum_balance(data.len())),
            data,
            &spl_token::ID,
            false,
            Epoch::default(),
        )
    }

    pub fn new_token_account<'a, 'b>(
        &self,
        mint_pubkey: &'a Pubkey,
        owner_pubkey: &'b Pubkey,
        balance: u64,
    ) -> AccountInfo {
        let rent = Rent::default();
        let data = self.storage.alloc_slice_fill_copy(TokenAccount::LEN, 0u8);
        let mut account = TokenAccount::default();
        account.state = spl_token::state::AccountState::Initialized;
        account.mint = *mint_pubkey;
        account.owner = *owner_pubkey;
        account.amount = balance;
        TokenAccount::pack(account, data).unwrap();
        AccountInfo::new(
            random_pubkey(&self.storage),
            false,
            true,
            self.storage.alloc(rent.minimum_balance(data.len())),
            data,
            &spl_token::ID,
            false,
            Epoch::default(),
        )
    }

    pub fn new_program(&self) -> AccountInfo {
        AccountInfo::new(
            random_pubkey(&self.storage),
            false,
            false,
            self.storage.alloc(0),
            &mut [],
            &bpf_loader::ID,
            true,
            Epoch::default(),
        )
    }

    fn new_rent_sysvar_account(&self) -> AccountInfo {
        let lamports = 100000;
        let data = self.storage.alloc_slice_fill_copy(size_of::<Rent>(), 0u8);
        let mut account_info = AccountInfo::new(
            &sysvar::rent::ID,
            false,
            false,
            self.storage.alloc(lamports),
            data,
            &sysvar::ID,
            false,
            Epoch::default(),
        );
        let rent = Rent::default();
        rent.to_account_info(&mut account_info).unwrap();
        account_info
    }
}

fn random_pubkey(bump: &Bump) -> &Pubkey {
    bump.alloc(Pubkey::new(transmute_to_bytes(&rand::random::<[u64; 4]>())))
}

// Program that can be executed in the environment.
pub trait Program: Send + Sync + Debug {
    // The program's ID.
    fn id(&self) -> Pubkey;

    // Entrypoint to start executing the program.
    fn entry(&self, program_id: &Pubkey, accounts: &[AccountInfo], ix_data: &[u8])
        -> ProgramResult;
}

#[derive(Debug)]
struct SplToken;

impl Program for SplToken {
    fn entry(
        &self,
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        ix_data: &[u8],
    ) -> ProgramResult {
        spl_token::processor::Processor::process(program_id, accounts, ix_data)
    }
    fn id(&self) -> Pubkey {
        spl_token::ID
    }
}

// Bare minimum implementation of the system program. Not all instructions are
// implemented. PRs are welcome.
#[derive(Debug)]
struct SystemProgram;

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
        let mut created = next_account_info(acc_infos)?;

        if **created.lamports.borrow() > 0 {
            panic!("{}", SystemError::AccountAlreadyInUse);
        }

        let environment = env();
        let data = environment
            .accounts
            .storage()
            .alloc_slice_fill_copy(space as usize, 0u8);
        // Safe because the lifetime is extended to match the other accounts
        // also allocated in the bump allocator.
        let data = unsafe { extend_lifetime(data) };

        let created = unsafe { into_mut(created) };
        created.data.replace(data);

        **from.lamports.borrow_mut() -= lamports;
        **created.lamports.borrow_mut() += lamports;

        Ok(())
    }

    fn transfer(&self, accounts: &[AccountInfo], lamports: u64) -> ProgramResult {
        let acc_infos = &mut accounts.into_iter();

        let from = next_account_info(acc_infos)?;
        let to = next_account_info(acc_infos)?;

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

impl Program for SystemProgram {
    fn entry(
        &self,
        program_id: &Pubkey,
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

unsafe fn extend_lifetime<'a, 'info>(data: &'a mut [u8]) -> &'info mut [u8] {
    std::mem::transmute::<&'a mut [u8], &'info mut [u8]>(data)
}

unsafe fn into_mut<'a, 'info>(acc: &'a AccountInfo<'info>) -> &'a mut AccountInfo<'info> {
    std::mem::transmute::<&AccountInfo, &mut AccountInfo>(acc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_program_creates_account() {
        let mut environment = Environment::new();

        let owner = Pubkey::new_unique();
        let from = environment.accounts.new_sol_account(1000);
        let created = environment.accounts.new_sol_account(0);

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
        environment.invoke(&ix, accounts, &[]).unwrap();
        assert_eq!(*created.data.borrow_mut(), &[0u8; 1234]);
        assert_eq!(**created.lamports.borrow(), 999);
        assert_eq!(**from.lamports.borrow(), 1);
    }

    #[test]
    fn system_program_transfer() {
        let mut environment = Environment::new();

        let owner = Pubkey::new_unique();
        let from = environment.accounts.new_sol_account(1000);
        let to = environment.accounts.new_sol_account(0);

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
        environment.invoke(&ix, accounts, &[]).unwrap();
        assert_eq!(**from.lamports.borrow(), 1);
        assert_eq!(**to.lamports.borrow(), 999);
    }
}
