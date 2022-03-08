//! This example tests the rendering of IDL JSON

use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

/// This comment should not end up in the IDL
#[program]
mod idl_parse_test {
    //! program description, should end up in IDL
    use super::*;

    /// This comment should *not* end up in the IDL.
    pub fn instruction_one(_ctx: Context<AccountsMetaOne>,
        /// var description
        var: u64,
        ) -> Result<()> {
        //! instruction_one description
        Ok(())
    }

    /// Hello.
    pub fn instruction_one(_ctx: Context<AccountsMetaOne>,
        /// var description
        var: u64,
        ) -> Result<()> {
        //! instruction_one description
        Ok(())
    }
}

#[derive(Accounts)]
pub struct AccountsMetaOne<'info> {
    //! AccountsMetaOne description
    /// Rent sysvar.
    /// Multi line docs.
    pub rent: Sysvar<'info, Rent>,
    /// Composite accounts test.
    /// Multiple lines supported.
    /// You can also include "double quotes".
    pub other: OtherAccount<'info>,
}

/// Hello accounts.
#[derive(Accounts)]
pub struct OtherAccount<'info> {
    //! OtherAccount description
    /// custom account description
    pub custom_account: Account<'info, CustomAccount>,
    
}

#[account]
pub struct CustomAccount {
    //! CustomAccount description

    /// describing how this field does things
    field_one: u64,
    /// and this field does things too
    field_two: String,
}

/// MyError.
#[error_code]
pub enum MyError {
    /// comment, not in IDL
    #[msg("This is an error message clients will automatically display")]
    BadThing,
    /// another comment, not in IDL
    OtherBadThing = 123,
}
