use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod anchor_cpi_return {
    use super::*;

    pub fn initialize_return(ctx: Context<InitializeReturn>) -> Result<u64> {
        Ok(10)
    }
}

#[derive(Accounts)]
pub struct InitializeReturn {}
