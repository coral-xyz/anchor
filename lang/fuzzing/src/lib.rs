//! The fuzz module provides utilities to facilitate fuzzing anchor programs.

#![allow(mutable_transmutes)]

use crate::spl_token_program::SplTokenProgram;
use crate::system_program::SystemProgram;
use bumpalo::Bump;
use safe_transmute::to_bytes::transmute_to_bytes;
use solana_program::account_info::AccountInfo;
use solana_program::bpf_loader;
use solana_program::clock::Epoch;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::Instruction;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::{self, Sysvar};
use spl_token::state::{Account as TokenAccount, Mint};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::mem::size_of;

mod spl_token_program;
mod system_program;

lazy_static::lazy_static! {
    static ref ENV: Host = Host::new();
}

// Global host environment.
pub fn env() -> &'static Host {
    &ENV
}

// The host execution environment.
#[derive(Debug)]
pub struct Host {
    // All registered programs that can be invoked.
    programs: HashMap<Pubkey, Box<dyn Program>>,
    // The currently executing program.
    current_program: RefCell<Option<Pubkey>>,
    // Account storage.
    accounts: AccountStore,
}

impl Host {
    pub fn new() -> Host {
        let mut env = Host {
            programs: HashMap::new(),
            current_program: RefCell::new(None),
            accounts: AccountStore::new(),
        };
        env.register(Box::new(SystemProgram));
        env.register(Box::new(SplTokenProgram));
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
                // If a PDA was given, mark it as signer.
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

// Not acutally Sync. Implemented so that we can use the Host as a
// lazy static without using locks (which is inconvenient and can cause
// deadlock). The Host, as presently constructed, should never be
// used across threads.
unsafe impl std::marker::Sync for Host {}

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
            // Allocate on the bump allocator, so that the owner can be safely
            // mutated by the SystemProgram's `create_account` instruction.
            self.storage.alloc(system_program::ID),
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
