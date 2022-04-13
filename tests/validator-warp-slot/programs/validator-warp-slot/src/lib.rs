use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod validator_warp_slot {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>, mainnet_slot: u64) -> Result<()> {
        // mainnet clock data is fetched after starting the validator; allow some tolerance
        let tolerance: u64 = 100;
        let test_validator_slot = Clock::get().unwrap().slot;
        assert!(mainnet_slot - test_validator_slot < tolerance);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
