use {
    crate::cursor::AccountCursor,
    pinocchio::address::Address,
    solana_program_error::ProgramError,
};

/// Instruction-scoped context passed to every handler. Holds the
/// declared accounts, program_id, PDA bumps, and a cursor for lazy
/// `remaining_accounts()` access.
pub struct Context<'a, T: Bumps> {
    /// Program id as a reference — lives for the whole instruction
    /// since it comes from the entrypoint's input buffer.
    pub program_id: &'a Address,

    /// Declared accounts (the `#[derive(Accounts)]` struct).
    pub accounts: T,

    /// Bump seeds found during constraint validation. Provided as a
    /// convenience so handlers don't have to recalculate bump seeds or
    /// pass them in as arguments.
    pub bumps: T::Bumps,

    /// Holds either a cursor into the serialized input buffer, pointing to the
    /// *start* of the remaining-accounts region (after `try_accounts`
    /// has consumed exactly `T::HEADER_SIZE` declared accounts). Used
    /// by [`Self::remaining_accounts`] for on-demand walking.
    /// After `remaining_accounts` is called this holds the range of views
    /// already written to the cursor's lookup table.
    remaining_accounts: RemainingAccounts,

    /// The instruction's account lookup table. Remaining accounts are exposed
    /// as a borrowed slice into this table instead of being copied into a
    /// second fixed-capacity buffer.
    cursor: &'a mut AccountCursor,

    /// Mutable-account mask covering active mutable accounts in the declared
    /// region. Starts from `T::MUT_MASK` and includes optional mutable fields
    /// only when the loaded account is `Some`. Used by
    /// [`Self::remaining_accounts`] to re-check each trailing account against
    /// declared mut slots. The `HEADER_SIZE` check in `run_handler` cannot see
    /// duplicates that only surface while walking the trailing region.
    mut_mask: MutMask,
}

pub enum MutMask {
    Static(&'static [u64; 4]),
    Dynamic([u64; 4]),
}

impl MutMask {
    #[inline(always)]
    fn as_ref(&self) -> &[u64; 4] {
        match self {
            Self::Static(mask) => mask,
            Self::Dynamic(mask) => mask,
        }
    }
}

enum RemainingAccounts {
    Unparsed {
        /// Number of accounts remaining after the initially declared region
        remaining: u8,
    },
    /// Cached range of views in the cursor's lookup table.
    Cached { start: u8, len: u8 },
    /// Cached error from walking `remaining-accounts`.
    Failed(ProgramError),
}

impl RemainingAccounts {
    fn is_unparsed(&self) -> bool {
        matches!(self, RemainingAccounts::Unparsed { .. })
    }
}

impl<'a, T: Bumps> Context<'a, T> {
    #[inline(always)]
    pub fn new(
        program_id: &'a Address,
        accounts: T,
        bumps: T::Bumps,
        cursor: &'a mut AccountCursor,
        remaining_num: u8,
        mut_mask: MutMask,
    ) -> Self {
        Self {
            program_id,
            accounts,
            bumps,
            remaining_accounts: RemainingAccounts::Unparsed {
                remaining: remaining_num,
            },
            cursor,
            mut_mask,
        }
    }

    /// Returns trailing accounts beyond the declared `T` fields as a borrowed
    /// slice into the instruction's account lookup table. The first call
    /// walks the cursor and caches the resulting range; subsequent calls
    /// replay that range. Caching the error is important: the walk advances
    /// an unsafe cursor, and a handler that calls this again after an error
    /// must not trigger another `cursor.next()` loop.
    ///
    /// After each cursor advance, re-tests the cursor's duplicate bitvec
    /// against the active mutable mask. If a trailing account's dup index
    /// resolves to a declared mut slot, returns
    /// `ConstraintDuplicateMutableAccount`. The `HEADER_SIZE`-only check
    /// in `run_handler` only sees duplicates that existed at the end of
    /// the declared walk; trailing-region dups can only be caught here.
    ///
    /// The mask is sized per declared field, so bits set for trailing indices
    /// (past `HEADER_SIZE`) are naturally zero — the intersect only fires when
    /// a trailing slot's bit overlaps with an active declared mut slot's bit,
    /// which by construction means the runtime resolved the trailing slot as a
    /// dup of that declared mut account.
    pub fn remaining_accounts(
        &mut self,
    ) -> Result<&'a [pinocchio::account::AccountView], ProgramError> {
        if self.remaining_accounts.is_unparsed() {
            self.remaining_accounts = match self.walk_remaining() {
                Ok((start, len)) => RemainingAccounts::Cached { start, len },
                Err(err) => RemainingAccounts::Failed(err),
            };
        }

        match &self.remaining_accounts {
            RemainingAccounts::Cached { start, len } => {
                // SAFETY: the dispatcher-owned lookup table lives for `'a`,
                // which is also the lifetime of this instruction context.
                Ok(unsafe { self.cursor.views(*start, *len) })
            }
            RemainingAccounts::Failed(err) => Err(err.clone()),
            RemainingAccounts::Unparsed { .. } => unreachable!(),
        }
    }

    fn walk_remaining(&mut self) -> Result<(u8, u8), ProgramError> {
        let remaining = match self.remaining_accounts {
            RemainingAccounts::Unparsed { remaining } => remaining,
            RemainingAccounts::Cached { .. } | RemainingAccounts::Failed(_) => unreachable!(),
        };
        let start = self.cursor.consumed();
        let mut_mask = self.mut_mask.as_ref();
        for _ in 0..remaining {
            // SAFETY: cursor is positioned at the start of the remaining
            // region and `remaining_num` is the exact number of accounts
            // to walk. Walking stops on the first mut-alias error so we
            // never advance past the remaining region.
            unsafe { self.cursor.next() };
            if let Some(dups) = self.cursor.duplicates() {
                if dups.intersects(mut_mask) {
                    return Err(crate::ErrorCode::ConstraintDuplicateMutableAccount.into());
                }
            }
        }
        Ok((start, remaining))
    }
}

/// Trait linking an accounts struct to its generated bumps struct.
pub trait Bumps {
    type Bumps;
}
