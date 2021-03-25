//! The fuzz modules provides utilities to facilitate fuzzing anchor programs.

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
use solana_program::system_program;
use solana_program::sysvar::{self, Sysvar};
use spl_token::state::Account as TokenAccount;
use spl_token::state::Mint;
use std::collections::HashMap;
use std::fmt::Debug;
use std::mem::size_of;
use std::sync::{Arc, Mutex, MutexGuard};

lazy_static::lazy_static! {
    static ref ENV: Arc<Mutex<Environment>> = Arc::new(Mutex::new(Environment::new()));
}

// Global host environment.
pub fn env() -> MutexGuard<'static, Environment> {
    ENV.lock().unwrap()
}

// The host execution environment.
pub struct Environment {
    // All registered programs that can be invoked.
    programs: HashMap<Pubkey, Box<dyn Program>>,
    // The currently executing program.
    current_program: Option<Pubkey>,
    // Account storage.
    accounts: AccountStore,
}

impl Environment {
    pub fn new() -> Environment {
        let mut env = Environment {
            programs: HashMap::new(),
            current_program: None,
            accounts: AccountStore::new(),
        };
        env.register(Box::new(SplToken));
        env
    }

    // Registers the program on the environment so that it can be invoked via
    // CPI.
    pub fn register(&mut self, program: Box<dyn Program>) {
        self.programs.insert(program.id(), program);
    }

    // Performs a cross program invocation.
    pub fn invoke<'info>(
        &mut self,
        ix: &Instruction,
        accounts: &[AccountInfo<'info>],
        seeds: &[&[&[u8]]],
    ) -> ProgramResult {
        let current_program = self.current_program.unwrap();

        // If seeds were given, then calculate the expected PDA.
        let pda = match seeds.len() > 0 {
            false => None,
            true => Some(Pubkey::create_program_address(seeds[0], &current_program).unwrap()),
        };

        self.current_program = Some(ix.program_id);
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

struct AccountStore {
    // Storage bytes.
    storage: Bump,
}

impl AccountStore {
    pub fn new() -> Self {
        Self {
            storage: Bump::new(),
        }
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
