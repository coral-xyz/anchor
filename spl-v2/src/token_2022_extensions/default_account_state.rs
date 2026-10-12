use {
    super::common::validate_token_2022_program,
    crate::{
        token_2022::spl_token_2022,
        token_shared::{token_data_subinstruction, TokenInstructionData},
    },
    anchor_lang::{CpiContext, CpiHandle, CpiHandleMut, ToCpiAccounts},
    solana_program_error::ProgramError,
};

#[derive(ToCpiAccounts)]
pub struct DefaultAccountStateInitialize<'a> {
    pub mint: CpiHandleMut<'a>,
}

#[derive(ToCpiAccounts)]
pub struct DefaultAccountStateUpdate<'a> {
    pub mint: CpiHandleMut<'a>,
    #[signer]
    pub freeze_authority: CpiHandle<'a>,
}

pub fn default_account_state_initialize<'a>(
    ctx: CpiContext<'a, DefaultAccountStateInitialize<'a>>,
    state: &spl_token_2022::state::AccountState,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let state = match state {
        spl_token_2022::state::AccountState::Uninitialized => 0,
        spl_token_2022::state::AccountState::Initialized => 1,
        spl_token_2022::state::AccountState::Frozen => 2,
    };
    let mut data: TokenInstructionData = token_data_subinstruction(28, 0);
    data.push(state);
    ctx.invoke(&data)
}

pub fn default_account_state_update<'a>(
    ctx: CpiContext<'a, DefaultAccountStateUpdate<'a>>,
    state: &spl_token_2022::state::AccountState,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let state = match state {
        spl_token_2022::state::AccountState::Uninitialized => 0,
        spl_token_2022::state::AccountState::Initialized => 1,
        spl_token_2022::state::AccountState::Frozen => 2,
    };
    let mut data: TokenInstructionData = token_data_subinstruction(28, 1);
    data.push(state);
    ctx.invoke_with_signer_override(&data, Some((1, ctx.remaining_accounts.is_empty())))
}
