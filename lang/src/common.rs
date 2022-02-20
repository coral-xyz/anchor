use crate::error::ErrorCode;
use solana_program::account_info::AccountInfo;
use std::io::Write;

pub fn close<'info>(
    info: AccountInfo<'info>,
    sol_destination: AccountInfo<'info>,
) -> anchor_lang::Result<()> {
    // Transfer tokens from the account to the sol_destination.
    let dest_starting_lamports = sol_destination.lamports();
    **sol_destination.lamports.borrow_mut() =
        dest_starting_lamports.checked_add(info.lamports()).unwrap();
    **info.lamports.borrow_mut() = 0;

    // Mark the account discriminator as closed.
    let mut data = info.try_borrow_mut_data()?;
    let dst: &mut [u8] = &mut data;
    let mut cursor = std::io::Cursor::new(dst);
    if cursor
        .write_all(&crate::__private::CLOSED_ACCOUNT_DISCRIMINATOR)
        .is_err()
    {
        return Err(ErrorCode::AccountDidNotSerialize.into());
    }
    Ok(())
}
