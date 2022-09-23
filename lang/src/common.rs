use crate::Result;
use solana_program::account_info::AccountInfo;
use solana_program::rent::Rent;
use solana_program::system_program;
use solana_program::sysvar::Sysvar;

/// This should now be safe to use anywhere
pub fn close<'info>(info: AccountInfo<'info>, sol_destination: AccountInfo<'info>) -> Result<()> {
    // Transfer tokens from the account to the sol_destination.
    let dest_starting_lamports = sol_destination.lamports();
    **sol_destination.lamports.borrow_mut() =
        dest_starting_lamports.checked_add(info.lamports()).unwrap();
    **info.lamports.borrow_mut() = 0;
    info.realloc(0, false)?;
    info.assign(&solana_program::system_program::ID);
    Ok(())
}

/// Not safe to use manually in an instruction's main function logic because at `exit` the discriminator will
/// be overwritten.
pub fn destroy<'info>(info: AccountInfo<'info>, sol_destination: AccountInfo<'info>) -> Result<()> {
    let anchor_rent = Rent::get()?;
    // Leave 8 bytes in the account to store the closed account discriminator permanently
    let eight_byte_rent = anchor_rent.minimum_balance(8);
    let lamport_amt = info.lamports().checked_sub(eight_byte_rent).unwrap();
    // Transfer tokens from the account to the sol_destination.
    let dest_starting_lamports = sol_destination.lamports();
    **sol_destination.lamports.borrow_mut() =
        dest_starting_lamports.checked_add(lamport_amt).unwrap();
    **info.lamports.borrow_mut() = eight_byte_rent;
    info.realloc(8, false)?;

    info.assign(&system_program::ID);
    info.realloc(0, false).map_err(Into::into)
}
