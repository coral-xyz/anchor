use {
    crate::{address_eq, require, Address, CpiHandle},
    core::{mem::MaybeUninit, slice::from_raw_parts},
    pinocchio::{
        cpi::{CpiAccount, Seed, Signer},
        instruction::{InstructionAccount, InstructionView},
    },
    solana_program_error::{ProgramError, ProgramResult},
};

#[cfg(not(target_os = "solana"))]
use solana_instruction::Instruction;

pub use pinocchio::cpi::{set_return_data, MAX_RETURN_DATA};

/// Get the return data from an invoked program.
///
/// This preserves the v1-shaped `program::get_return_data()` return value while
/// sourcing the data from Pinocchio's `AccountView`-native CPI module.
///
/// The returned Pinocchio buffer is bounded by Solana's 1 KiB return-data
/// limit, avoiding a second large stack allocation on SBF. Callers that need
/// an owned instruction-data buffer should copy `ReturnData::as_slice()` into
/// their own bounded buffer.
pub fn get_return_data() -> Option<(crate::Address, pinocchio::cpi::ReturnData)> {
    pinocchio::cpi::get_return_data().map(|data| (*data.program_id(), data))
}

/// Invoke a cross-program instruction using Anchor v2 CPI handles.
///
/// Unlike the legacy `AccountInfo` API, callers pass [`CpiHandle`]s obtained
/// from `cpi_handle()` / `cpi_handle_mut()`, so CPI account lifetimes remain
/// tied to Rust borrows of the caller's typed accounts.
///
/// Optional program-id sentinel metas from `Option::None` CPI slots are not
/// inferred from address alone — use
/// [`invoke_with_optional_sentinels`] / [`invoke_signed_with_optional_sentinels`]
/// (or [`crate::CpiContext`]) so required readonly program-id accounts still
/// require matching handles.
#[cfg(not(target_os = "solana"))]
pub fn invoke<'a>(instruction: &Instruction, account_handles: &[CpiHandle<'a>]) -> ProgramResult {
    invoke_signed(instruction, account_handles, &[])
}

/// Invoke a borrowed instruction view using borrowed account handles.
///
/// This is the allocator-free CPI entry point. It accepts Pinocchio's
/// borrowed instruction representation directly, validates account ordering,
/// writability, signer requirements, and borrow state, then uses only
/// fixed-capacity stack buffers for the syscall arguments.
#[inline]
pub fn invoke_view<'a, 'b, 'c, 'd, 'seeds>(
    instruction: &InstructionView<'a, 'b, 'c, 'd>,
    account_handles: &[CpiHandle<'a>],
    signer_seeds: &'seeds [&'seeds [&'seeds [u8]]],
) -> ProgramResult {
    let empty_flags: &[bool] = &[];
    invoke_view_with_optional_sentinels(instruction, account_handles, signer_seeds, empty_flags)
}

/// Borrowed CPI entry point with an explicit mask for optional program-id
/// sentinel accounts.
#[inline]
pub fn invoke_view_with_optional_sentinels<'a, 'b, 'c, 'd, 'seeds, F>(
    instruction: &InstructionView<'a, 'b, 'c, 'd>,
    account_handles: &[CpiHandle<'a>],
    signer_seeds: &'seeds [&'seeds [&'seeds [u8]]],
    optional_sentinel_flags: &F,
) -> ProgramResult
where
    F: crate::OptionalSentinelFlags + ?Sized,
{
    validate_instruction_accounts(
        instruction.accounts,
        instruction.program_id,
        account_handles,
        optional_sentinel_flags,
        signer_seeds.is_empty(),
    )?;

    let cpi_account_count = required_cpi_account_count(
        instruction.accounts,
        instruction.program_id,
        account_handles,
        optional_sentinel_flags,
    )?;
    let _borrow_guards = crate::enter_cpi(account_handles);
    let cpi_accounts = cpi_accounts(account_handles);
    let cpi_accounts = unsafe {
        from_raw_parts(
            cpi_accounts.as_ptr() as *const CpiAccount,
            cpi_account_count,
        )
    };

    // SAFETY: validation proves that the handles correspond to the non-
    // sentinel instruction accounts, and every CpiAccount was initialized
    // from its corresponding AccountView before the syscall.
    match signer_seeds {
        [] => unsafe {
            pinocchio::cpi::invoke_signed_unchecked(instruction, cpi_accounts, &[]);
        },
        [seeds] => {
            let signer = signer_from_seeds(seeds);
            unsafe {
                pinocchio::cpi::invoke_signed_unchecked(
                    instruction,
                    cpi_accounts,
                    &[signer],
                );
            }
        }
        signer_seeds => {
            let signers = signers(signer_seeds);
            unsafe {
                pinocchio::cpi::invoke_signed_unchecked(instruction, cpi_accounts, &signers);
            }
        }
    }

    Ok(())
}

/// Like [`invoke`], but marks instruction-account indices that are intentional
/// optional `None` program-id sentinels (parallel to
/// [`crate::ToCpiAccounts::optional_account_sentinel_flags`]).
#[cfg(not(target_os = "solana"))]
pub fn invoke_with_optional_sentinels<'a, F>(
    instruction: &Instruction,
    account_handles: &[CpiHandle<'a>],
    optional_sentinel_flags: &F,
) -> ProgramResult
where
    F: crate::OptionalSentinelFlags + ?Sized,
{
    invoke_signed_with_optional_sentinels(
        instruction,
        account_handles,
        &[],
        optional_sentinel_flags,
    )
}

/// Invoke a cross-program instruction with PDA signer seeds using Anchor v2
/// CPI handles.
#[cfg(not(target_os = "solana"))]
pub fn invoke_signed<'a, 'seeds>(
    instruction: &Instruction,
    account_handles: &[CpiHandle<'a>],
    signer_seeds: &'seeds [&'seeds [&'seeds [u8]]],
) -> ProgramResult {
    let empty_flags: &[bool] = &[];
    invoke_signed_with_optional_sentinels(instruction, account_handles, signer_seeds, empty_flags)
}

/// Like [`invoke_signed`], with an explicit optional-sentinel mask.
#[cfg(not(target_os = "solana"))]
pub fn invoke_signed_with_optional_sentinels<'a, 'seeds, F>(
    instruction: &Instruction,
    account_handles: &[CpiHandle<'a>],
    signer_seeds: &'seeds [&'seeds [&'seeds [u8]]],
    optional_sentinel_flags: &F,
) -> ProgramResult
where
    F: crate::OptionalSentinelFlags + ?Sized,
{
    let instruction_accounts = instruction_accounts(instruction);
    validate_instruction_accounts(
        &instruction_accounts,
        &instruction.program_id,
        account_handles,
        optional_sentinel_flags,
        signer_seeds.is_empty(),
    )?;

    // SAFETY: Validation above proves every non-sentinel instruction account
    // has a matching handle, writable metas use writable handles, and
    // AccountView borrow state permits the CPI.
    unsafe {
        invoke_signed_unchecked_with_optional_sentinels(
            instruction,
            account_handles,
            signer_seeds,
            optional_sentinel_flags,
        )
    }
}

/// Invoke a cross-program instruction without borrow validation.
///
/// # Safety
///
/// The caller must ensure no live Rust references or stale `AccountView` data
/// borrows can be invalidated by the callee. Prefer [`invoke`] unless this is
/// being used through a higher-level API that already enforces those lifetimes.
#[cfg(not(target_os = "solana"))]
pub unsafe fn invoke_unchecked<'a>(
    instruction: &Instruction,
    account_handles: &[CpiHandle<'a>],
) -> ProgramResult {
    unsafe { invoke_signed_unchecked(instruction, account_handles, &[]) }
}

/// Invoke a cross-program instruction with PDA signer seeds, without borrow
/// validation.
///
/// # Safety
///
/// The caller must ensure no live Rust references or stale `AccountView` data
/// borrows can be invalidated by the callee. Prefer [`invoke_signed`] unless
/// this is being used through a higher-level API that already enforces those
/// lifetimes.
#[cfg(not(target_os = "solana"))]
pub unsafe fn invoke_signed_unchecked<'a, 'seeds>(
    instruction: &Instruction,
    account_handles: &[CpiHandle<'a>],
    signer_seeds: &'seeds [&'seeds [&'seeds [u8]]],
) -> ProgramResult {
    let empty_flags: &[bool] = &[];
    unsafe {
        invoke_signed_unchecked_with_optional_sentinels(
            instruction,
            account_handles,
            signer_seeds,
            empty_flags,
        )
    }
}

/// Like [`invoke_signed_unchecked`], with an explicit optional-sentinel mask.
///
/// # Safety
///
/// Same as [`invoke_signed_unchecked`].
#[cfg(not(target_os = "solana"))]
pub unsafe fn invoke_signed_unchecked_with_optional_sentinels<'a, 'seeds, F>(
    instruction: &Instruction,
    account_handles: &[CpiHandle<'a>],
    signer_seeds: &'seeds [&'seeds [&'seeds [u8]]],
    optional_sentinel_flags: &F,
) -> ProgramResult
where
    F: crate::OptionalSentinelFlags + ?Sized,
{
    let instruction_accounts = instruction_accounts(instruction);
    let cpi_account_count = required_cpi_account_count(
        &instruction_accounts,
        &instruction.program_id,
        account_handles,
        optional_sentinel_flags,
    )?;
    let instruction_view = InstructionView {
        program_id: &instruction.program_id,
        accounts: &instruction_accounts,
        data: &instruction.data,
    };
    let _borrow_guards = crate::enter_cpi(account_handles);
    let cpi_accounts = cpi_accounts(account_handles);
    let cpi_accounts = unsafe {
        from_raw_parts(
            cpi_accounts.as_ptr() as *const CpiAccount,
            cpi_account_count,
        )
    };

    // SAFETY:
    // - `cpi_accounts` was fully initialized from the provided handles.
    // - This function's caller upholds the unchecked CPI aliasing contract.
    match signer_seeds {
        [] => unsafe {
            pinocchio::cpi::invoke_signed_unchecked(&instruction_view, cpi_accounts, &[]);
        },
        [seeds] => {
            let signer = signer_from_seeds(seeds);
            unsafe {
                pinocchio::cpi::invoke_signed_unchecked(
                    &instruction_view,
                    cpi_accounts,
                    &[signer],
                );
            }
        }
        signer_seeds => {
            let signers = signers(signer_seeds);
            unsafe {
                pinocchio::cpi::invoke_signed_unchecked(
                    &instruction_view,
                    cpi_accounts,
                    &signers,
                );
            }
        }
    }

    Ok(())
}

pub(crate) fn validate_fixed_instruction_accounts<'a, const N: usize>(
    instruction_accounts: &[InstructionAccount<'a>; N],
    handles: &[CpiHandle<'a>; N],
    enforce_signers: bool,
) -> ProgramResult {
    for (account, handle) in instruction_accounts.iter().zip(handles) {
        require!(
            address_eq(account.address, handle.address())
                && (!account.is_writable || handle.is_writable()),
            ProgramError::InvalidArgument
        );

        if handle.requires_borrow_check() {
            if account.is_writable {
                handle.account_view().check_borrow_mut()?;
            } else {
                handle.account_view().check_borrow()?;
            }
        }

        if enforce_signers && account.is_signer {
            require!(
                handle.account_view().is_signer(),
                ProgramError::MissingRequiredSignature
            );
        }
    }

    Ok(())
}

pub(crate) fn validate_instruction_accounts<'a, F>(
    instruction_accounts: &[InstructionAccount<'a>],
    program_id: &Address,
    account_handles: &[CpiHandle<'a>],
    optional_sentinel_flags: &F,
    enforce_signers: bool,
) -> ProgramResult
where
    F: crate::OptionalSentinelFlags + ?Sized,
{
    let bindings = resolve_instruction_account_bindings(
        instruction_accounts,
        program_id,
        account_handles,
        optional_sentinel_flags,
    )?;

    for (account, binding) in instruction_accounts.iter().zip(bindings.iter()) {
        let Some(handle_index) = binding else {
            continue;
        };
        let handle = &account_handles[*handle_index];

        if account.is_writable {
            if handle.requires_borrow_check() {
                handle.account_view().check_borrow_mut()?;
            }
        } else if handle.requires_borrow_check() {
            handle.account_view().check_borrow()?;
        }

        if enforce_signers && account.is_signer {
            require!(
                handle.account_view().is_signer(),
                ProgramError::MissingRequiredSignature
            );
        }
    }

    Ok(())
}

fn is_optional_instruction_account_sentinel_candidate(
    program_id: &Address,
    account: &InstructionAccount<'_>,
) -> bool {
    !account.is_writable && !account.is_signer && address_eq(account.address, program_id)
}

fn is_skippable_optional_sentinel<F: crate::OptionalSentinelFlags + ?Sized>(
    program_id: &Address,
    account: &InstructionAccount<'_>,
    optional_sentinel_flags: &F,
    instruction_index: usize,
) -> bool {
    optional_sentinel_flags.get(instruction_index)
        && is_optional_instruction_account_sentinel_candidate(program_id, account)
}

fn required_cpi_account_count<'a, F: crate::OptionalSentinelFlags + ?Sized>(
    instruction_accounts: &[InstructionAccount<'a>],
    program_id: &Address,
    account_handles: &[CpiHandle<'a>],
    optional_sentinel_flags: &F,
) -> Result<usize, ProgramError> {
    Ok(resolve_instruction_account_bindings(
        instruction_accounts,
        program_id,
        account_handles,
        optional_sentinel_flags,
    )?
    .iter()
    .flatten()
    .count())
}

fn resolve_instruction_account_bindings<'a, F: crate::OptionalSentinelFlags + ?Sized>(
    instruction_accounts: &[InstructionAccount<'a>],
    program_id: &Address,
    account_handles: &[CpiHandle<'a>],
    optional_sentinel_flags: &F,
) -> Result<crate::FixedVec<Option<usize>, { crate::MAX_ACCOUNT_METAS }>, ProgramError> {
    let mut bindings = crate::FixedVec::new();
    let mut handle_index = 0;

    for (instruction_index, account) in instruction_accounts.iter().enumerate() {
        let can_skip = is_skippable_optional_sentinel(
            program_id,
            account,
            optional_sentinel_flags,
            instruction_index,
        ) && (handle_index >= account_handles.len()
            || instruction_accounts[instruction_index + 1..]
                .iter()
                .enumerate()
                .any(|(offset, later)| {
                    let later_index = instruction_index + 1 + offset;
                    !is_skippable_optional_sentinel(
                        program_id,
                        later,
                        optional_sentinel_flags,
                        later_index,
                    ) && instruction_account_matches_handle(later, &account_handles[handle_index])
                }));
        if can_skip {
            bindings.push(None);
            continue;
        }

        let can_consume = handle_index < account_handles.len()
            && instruction_account_matches_handle(account, &account_handles[handle_index]);
        if can_consume {
            bindings.push(Some(handle_index));
            handle_index += 1;
            continue;
        }

        return Err(if handle_index < account_handles.len() {
            // A handle is available, but it does not match this instruction
            // account's address or mutability. This is an invalid CPI shape,
            // not a missing account.
            ProgramError::InvalidArgument
        } else {
            ProgramError::NotEnoughAccountKeys
        });
    }

    if handle_index != account_handles.len() {
        return Err(ProgramError::InvalidArgument);
    }

    Ok(bindings)
}

fn instruction_account_matches_handle(
    account: &InstructionAccount<'_>,
    handle: &CpiHandle<'_>,
) -> bool {
    address_eq(account.address, handle.address()) && (!account.is_writable || handle.is_writable())
}

#[cfg(not(target_os = "solana"))]
fn instruction_accounts(instruction: &Instruction) -> crate::InstructionAccountBuffer<'_> {
    let mut accounts = crate::InstructionAccountBuffer::new();
    for meta in &instruction.accounts {
        accounts.push(InstructionAccount::new(
            &meta.pubkey,
            meta.is_writable,
            meta.is_signer,
        ));
    }
    accounts
}

fn signers<'seeds>(
    signer_seeds: &'seeds [&'seeds [&'seeds [u8]]],
) -> crate::FixedVec<Signer<'seeds, 'seeds>, { crate::MAX_ACCOUNT_METAS }> {
    let mut out = crate::FixedVec::new();
    for seeds in signer_seeds {
        out.push(signer_from_seeds(seeds));
    }
    out
}

#[inline(always)]
fn signer_from_seeds<'seeds>(seeds: &'seeds [&'seeds [u8]]) -> Signer<'seeds, 'seeds> {
    // SAFETY: `Seed` has the same in-memory representation as `&[u8]`.
    let cpi_seeds: &[Seed] =
        unsafe { from_raw_parts(seeds.as_ptr() as *const Seed, seeds.len()) };
    Signer::from(cpi_seeds)
}

fn cpi_accounts<'a>(
    account_handles: &[CpiHandle<'a>],
) -> crate::FixedVec<MaybeUninit<CpiAccount<'a>>, { crate::MAX_ACCOUNT_METAS }> {
    let mut accounts = crate::FixedVec::new();
    for handle in account_handles {
        let mut slot = MaybeUninit::uninit();
        CpiAccount::init_from_account_view(handle.account_view(), &mut slot);
        accounts.push(slot);
    }

    accounts
}
