use crate::error::ErrorCode;
use crate::prelude::error;
use crate::Result;
use solana_program::account_info::AccountInfo;
use std::io::Write;

/// When used by a user inside their program,
/// this function should only be used with
/// the AccountInfo or UncheckedAccount types.
///
/// Do NOT use this function with other types.
///
/// Details: Using `close` with types like `Account<'info, T>` is not safe because
/// it requires the `mut` constraint but for that type the constraint
/// overwrites the "closed account" discriminator at the end of the instruction.
pub fn close<'info>(info: AccountInfo<'info>, sol_destination: AccountInfo<'info>) -> Result<()> {
    // Transfer tokens from the account to the sol_destination.
    let dest_starting_lamports = sol_destination.lamports();
    **sol_destination.lamports.borrow_mut() =
        dest_starting_lamports.checked_add(info.lamports()).unwrap();
    **info.lamports.borrow_mut() = 0;

    // Mark the account discriminator as closed.
    let mut data = info.try_borrow_mut_data()?;
    let dst: &mut [u8] = &mut data;
    let mut cursor = std::io::Cursor::new(dst);
    cursor
        .write_all(&crate::__private::CLOSED_ACCOUNT_DISCRIMINATOR)
        .map_err(|_| error!(ErrorCode::AccountDidNotSerialize))
}
