use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey, system_program,
};
pub fn close<'info>(
    info: AccountInfo<'info>,
    sol_destination: AccountInfo<'info>,
) -> ProgramResult {
    // Transfer tokens from the account to the sol_destination.
    let dest_starting_lamports = sol_destination.lamports();
    **sol_destination.lamports.borrow_mut() =
        dest_starting_lamports.checked_add(info.lamports()).unwrap();
    **info.lamports.borrow_mut() = 0;

    // Zero the buffer so that the owner can be reassigned.
    let mut data = info.try_borrow_mut_data()?;

    const ZEROS_LEN: usize = 1024;
    static ZEROS: [u8; ZEROS_LEN] = [0; ZEROS_LEN];
    let chunks = data.chunks_exact_mut(ZEROS_LEN);
    // Under the new bpf-tools, this turns into a sol_memcpy
    chunks.for_each(|chunk| chunk.copy_from_slice(&ZEROS));
    let chunks = data.chunks_exact_mut(ZEROS_LEN);
    let remainder = chunks.into_remainder();
    let remainder_len = remainder.len();
    remainder.copy_from_slice(&ZEROS[..remainder_len]);

    // Reassign address to the system program so it cannot be hijacked

    // This is safe since
    // 1. bpf VM is single-threaded
    // 2. KeyedAccount's pubkey is serialized to BPF's INPUT buffer per VM
    unsafe {
        let const_ptr = info.owner as *const Pubkey;
        let mut_ptr = const_ptr as *mut Pubkey;
        *mut_ptr = system_program::id();
    }
    assert_eq!(*info.owner, system_program::id());
    Ok(())
}
