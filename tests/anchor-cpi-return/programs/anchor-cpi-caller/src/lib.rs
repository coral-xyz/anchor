use anchor_lang::prelude::*;

declare_id!("HmbTLCmaGvZhKnn1Zfa1JVnp7vkMV4DYVxPLWBVoN65L");

#[program]
pub mod anchor_cpi_caller {
    use super::*;

    pub fn initialize_caller(ctx: Context<InitializeCaller>) -> Result<()> {
        let data = anchor_cpi_return::cpi::initialize_return(ctx);
        msg!(data);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeCaller {}
