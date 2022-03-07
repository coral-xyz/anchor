use anchor_lang::prelude::*;
use anchor_cpi_return::*

declare_id!("HmbTLCmaGvZhKnn1Zfa1JVnp7vkMV4DYVxPLWBVoN65L");

#[program]
pub mod anchor_cpi_caller {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let data = anchor_cpi_return::cpi::initialize(ctx);
        msg!(data);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
