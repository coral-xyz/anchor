use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod idl_descriptions {
    use super::*;

    /// Initialize the program.
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    /// Initialize
    /// Multi
    /// Line.
    pub fn initialize_multiline_doc(ctx: Context<InitializeMultilineDoc>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct InitializeMultilineDoc {}
