use anchor_lang::prelude::*;

declare_id!("Cum9tTyj5HwcEiAmhgaS7Bbj4UczCwsucrCkxRECzM4e");

// TODO: Once anchor can deserialize program data, update this test.
// Add constraint that program.program_data_address == program_data.key()

#[program]
pub mod program_data {
    use super::*;
    pub fn set_admin_settings(ctx: Context<SetAdminSettings>, admin_data: u64) -> ProgramResult {
        ctx.accounts.settings.admin_data = admin_data;
        Ok(())
    }
}

#[account]
#[derive(Default, Debug)]
pub struct Settings {
    admin_data: u64
}

#[derive(Accounts)]
pub struct SetAdminSettings<'info> {
    #[account(init, payer = authority)]
    pub settings: Account<'info, Settings>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(constraint = *program_data.upgrade_authority_address() == Some(authority.key()))]
    pub program_data: ProgramData<'info>,
    pub system_program: Program<'info, System>
}
