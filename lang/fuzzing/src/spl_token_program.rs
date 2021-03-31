use crate::Program;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::pubkey::Pubkey;
use std::fmt::Debug;

#[derive(Debug)]
pub struct SplTokenProgram;

impl Program for SplTokenProgram {
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
