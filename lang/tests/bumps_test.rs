use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct BumpsTest<'info> {
    #[account(
        seeds = [&[1u8]],
        bump,
    )]
    pub is_pda: SystemAccount<'info>,
    pub non_pda: SystemAccount<'info>,
}
