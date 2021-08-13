use crate::error::ErrorCode;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey, system_program,
};
use std::io::Write;

pub fn close<'info>(
    info: AccountInfo<'info>,
    sol_destination: AccountInfo<'info>,
) -> ProgramResult {
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
        .map_err(|_| ErrorCode::AccountDidNotSerialize)?;

    // Reassign address to the system program so it cannot be hijacked

    // This is safe since
    // 1. bpf VM is single-threaded
    // 2. KeyedAccount's pubkey is serialized to BPF's INPUT buffer per VM
    unsafe {
        let const_ptr = info.owner as *const Pubkey;
        let mut_ptr = const_ptr as *mut Pubkey;
        *mut_ptr = system_program::id();
    }
    Ok(())
}
