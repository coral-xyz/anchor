use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct GenericsTest<'info, T>
where
    T: AccountSerialize + AccountDeserialize + Clone,
{
    non_generic: AccountInfo<'info>,
    generic: ProgramAccount<'info, T>,
}
