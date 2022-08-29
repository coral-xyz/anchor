use crate::bpf_writer::BpfWriter;
use crate::error::ErrorCode;
use crate::prelude::error;
use crate::Result;
use solana_program::account_info::AccountInfo;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use std::io::Write;

pub fn close<'info>(info: AccountInfo<'info>, sol_destination: AccountInfo<'info>) -> Result<()> {
    // Transfer tokens from the account to the sol_destination.
    let dest_starting_lamports = sol_destination.lamports();
    **sol_destination.lamports.borrow_mut() =
        dest_starting_lamports.checked_add(info.lamports()).unwrap();
    **info.lamports.borrow_mut() = 0;
    info.realloc(0, false)?;
    info.assign(&solana_program::system_program::ID);
    Ok(())

    // Mark the account discriminator as closed.
    // let mut data = info.try_borrow_mut_data()?;
    // let dst: &mut [u8] = &mut data;
    // let mut writer = BpfWriter::new(dst);
    // writer
    //     .write_all(&crate::__private::CLOSED_ACCOUNT_DISCRIMINATOR)
    //     .map_err(|_| error!(ErrorCode::AccountDidNotSerialize))
}

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

    // Mark the account discriminator as closed.
    let mut data = info.try_borrow_mut_data()?;
    let dst: &mut [u8] = &mut data;
    let mut writer = BpfWriter::new(dst);
    writer
        .write_all(&crate::__private::CLOSED_ACCOUNT_DISCRIMINATOR)
        .map_err(|_| error!(ErrorCode::AccountDidNotSerialize))
}
