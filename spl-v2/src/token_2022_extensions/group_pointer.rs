use {
    super::common::validate_token_2022_program,
    crate::{
        token_shared::{
            push_optional_zero_address, token_data_subinstruction, TokenInstructionData,
        },
    },
    anchor_lang::{CpiContext, CpiHandle, CpiHandleMut, ToCpiAccounts},
    pinocchio::address::Address,
    solana_program_error::ProgramError,
};

#[derive(ToCpiAccounts)]
pub struct GroupPointerInitialize<'a> {
    pub mint: CpiHandleMut<'a>,
}

#[derive(ToCpiAccounts)]
pub struct GroupPointerUpdate<'a> {
    pub mint: CpiHandleMut<'a>,
    #[signer]
    pub authority: CpiHandle<'a>,
}

pub fn group_pointer_initialize<'a>(
    ctx: CpiContext<'a, GroupPointerInitialize<'a>>,
    authority: Option<&Address>,
    group_address: Option<&Address>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let mut data: TokenInstructionData = token_data_subinstruction(40, 0);
    push_optional_zero_address(&mut data, authority);
    push_optional_zero_address(&mut data, group_address);
    ctx.invoke(&data)
}

pub fn group_pointer_update<'a>(
    ctx: CpiContext<'a, GroupPointerUpdate<'a>>,
    group_address: Option<&Address>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let mut data: TokenInstructionData = token_data_subinstruction(40, 1);
    push_optional_zero_address(&mut data, group_address);
    ctx.invoke_with_signer_override(&data, Some((1, ctx.remaining_accounts.is_empty())))
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        anchor_lang::{
            testing::{AccountBuffer, MIN_ACCOUNT_BUF},
        },
    };

    fn account(
        address: [u8; 32],
        signer: bool,
        writable: bool,
    ) -> AccountBuffer<{ MIN_ACCOUNT_BUF + 8 }> {
        let buffer = AccountBuffer::new();
        buffer.init(address, [9; 32], 8, signer, writable, false);
        buffer
    }

    #[test]
    fn group_pointer_update_accounts_do_not_duplicate_authority() {
        let mint_buffer = account([1; 32], false, true);
        let authority_buffer = account([2; 32], true, false);
        let mut mint_view = unsafe { mint_buffer.view() };
        let authority_view = unsafe { authority_buffer.view() };

        let accounts = GroupPointerUpdate {
            mint: CpiHandleMut::writable(&mut mint_view),
            authority: CpiHandle::readonly(&authority_view),
        };

        assert_eq!(accounts.to_cpi_handles().len(), 2);
    }
}
