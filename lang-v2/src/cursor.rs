//! Lazy cursor over the serialized instruction input buffer.
//!
//! [`AccountCursor`] yields `AccountView`s on demand so dispatch can
//! happen first and each arm walks only its declared accounts. The runtime
//! path resolves duplicate account references by rescanning the compact
//! serialized input, so the entrypoint does not reserve a 256-element
//! `AccountView` lookup array on the stack. The legacy lookup-backed path is
//! retained for low-level callers and existing tests.

use {
    core::mem::MaybeUninit,
    pinocchio::account::{AccountView, RuntimeAccount, MAX_PERMITTED_DATA_INCREASE},
};

/// Sentinel value in the serialized `borrow_state` byte indicating a
/// non-duplicated account. Any other value (0..=254) indicates a
/// duplicate and holds the index of the earlier account it aliases.
pub const NON_DUP_MARKER: u8 = u8::MAX;

/// Static (fixed) per-account size in the serialized input buffer: the
/// `RuntimeAccount` header + the `MAX_PERMITTED_DATA_INCREASE` padding
/// region that trails the account data.
const STATIC_ACCOUNT_DATA: usize =
    core::mem::size_of::<RuntimeAccount>() + MAX_PERMITTED_DATA_INCREASE;

/// 8-byte alignment required between account records on BPF.
const BPF_ALIGN_OF_U128: usize = 8;

/// Cursor into the serialized instruction input buffer.
///
/// Advances a raw pointer past one account record per [`next`](AccountCursor::next)
/// call and uses a `lookup` array for dup resolution.
///
/// # Safety
///
/// Created from the runtime's input pointer (r1). Must not outlive the
/// entrypoint invocation. Callers must ensure:
///
/// - The lookup pointer passed to [`AccountCursor::new`] points to
///   `[AccountView; N]` where `N >= max(consumed + 1, max_dup_index + 1)`.
///   The runtime entrypoint should use [`AccountCursor::new_without_lookup`]
///   instead, which avoids this stack allocation.
/// - `next()` is called fewer than `num_accounts` times.
pub struct AccountCursor {
    /// Current position in the input buffer. Advances on each `next()`.
    ptr: *mut u8,

    /// Pointer to the caller's `[AccountView; N]` lookup array.
    /// Indexed by `consumed` on write and by the serialized dup index on read
    /// (for duplicate resolution). A null pointer selects the compact
    /// input-rescanning path used by generated entrypoints.
    lookup: *mut AccountView,

    /// Start of the serialized account records. Used by the compact duplicate
    /// resolver when `lookup` is null.
    input_start: *mut u8,

    /// Number of accounts yielded so far. Used both as the write index
    /// into `lookup` and as a runtime counter exposed to callers for
    /// bookkeeping (e.g., remaining-accounts walks).
    consumed: u8,

    /// Tracks accounts that are duplicates; the first instance of a duplicated account
    /// will also be marked.
    /// Lazy: stays `None` for txs with no duplicates (the common case),
    /// materialized via `get_or_insert_with` on the first dup seen.
    duplicate: Option<AccountBitvec>,
}

impl AccountCursor {
    /// Create a fresh cursor at the start of the serialized accounts
    /// region. `input_ptr` must point at the 8-byte `num_accounts`
    /// length prefix in the input buffer (i.e. the runtime-provided `r1`
    /// value); the cursor advances past it internally.
    ///
    /// # Safety
    ///
    /// See type-level safety notes.
    #[inline(always)]
    pub unsafe fn new(input_ptr: *mut u8, lookup: *mut AccountView) -> Self {
        let input_start = input_ptr.add(core::mem::size_of::<u64>());
        Self {
            ptr: input_start,
            lookup,
            input_start,
            consumed: 0,
            duplicate: None,
        }
    }

    /// Create a cursor without reserving a transaction-wide account lookup
    /// table. Duplicate records are resolved by scanning the serialized input
    /// only when a duplicate is encountered. This keeps the common no-dup path
    /// allocation-free and bounds the entrypoint frame by the current
    /// instruction's declared account count instead of 256 accounts.
    #[inline(always)]
    pub unsafe fn new_without_lookup(input_ptr: *mut u8) -> Self {
        Self::new(input_ptr, core::ptr::null_mut())
    }

    /// Number of accounts yielded from this cursor so far.
    #[inline(always)]
    pub fn consumed(&self) -> u8 {
        self.consumed
    }

    /// Current duplicate-tracking bitvec. `None` if the cursor has not
    /// yet yielded a duplicate account (lazy allocation — see
    /// [`Self::next`]). Used by
    /// [`Context::remaining_accounts`](crate::context::Context::remaining_accounts)
    /// to re-check `MUT_MASK` after each trailing account is walked,
    /// catching aliases of declared mut accounts that only surface past
    /// `HEADER_SIZE`.
    #[inline(always)]
    pub fn duplicates(&self) -> Option<&AccountBitvec> {
        self.duplicate.as_ref()
    }

    /// Walk N accounts in a tight loop, storing views in the lookup array.
    /// Returns a slice of the walked views and the duplicate tracking bitvec.
    /// This avoids interleaving cursor math with validation logic, letting
    /// LLVM optimize the walk loop.
    ///
    /// # Safety
    ///
    /// Caller must ensure N does not exceed the remaining accounts.
    #[inline(always)]
    pub unsafe fn walk_n(&mut self, n: usize) -> (&[AccountView], Option<&AccountBitvec>) {
        debug_assert!(!self.lookup.is_null());
        let start = self.consumed as usize;
        for _ in 0..n {
            self.next();
        }
        (
            core::slice::from_raw_parts(self.lookup.add(start), n),
            self.duplicate.as_ref(),
        )
    }

    /// Walk N accounts into caller-provided storage and return the initialized
    /// view slice. Generated dispatch uses this method so the storage is sized
    /// to the instruction's flattened account header rather than the maximum
    /// transaction account count.
    ///
    /// # Safety
    ///
    /// The output slice must have capacity for at least `n` views, and the
    /// cursor must have at least `n` account records remaining.
    #[inline(always)]
    pub unsafe fn walk_n_into<'views>(
        &mut self,
        n: usize,
        output: &'views mut [MaybeUninit<AccountView>],
    ) -> (&'views [AccountView], Option<&AccountBitvec>) {
        debug_assert!(output.len() >= n);
        for slot in output.iter_mut().take(n) {
            slot.write(self.next());
        }
        (
            core::slice::from_raw_parts(output.as_ptr() as *const AccountView, n),
            self.duplicate.as_ref(),
        )
    }

    /// Locate the raw account record for a serialized slot. Duplicate slots
    /// occupy only the eight-byte duplicate header, while ordinary records
    /// include their data and alignment padding.
    #[inline(always)]
    unsafe fn raw_account_at(&self, index: u8) -> *mut RuntimeAccount {
        let mut ptr = self.input_start;
        let mut current = 0u8;
        while current < index {
            let account = ptr as *mut RuntimeAccount;
            let borrow_state = (*account).borrow_state;
            ptr = ptr.add(core::mem::size_of::<u64>());
            if borrow_state == NON_DUP_MARKER {
                ptr = ptr.add(STATIC_ACCOUNT_DATA + (*account).data_len as usize);
                let addr = ptr.addr();
                let aligned = (addr + (BPF_ALIGN_OF_U128 - 1)) & !(BPF_ALIGN_OF_U128 - 1);
                ptr = ptr.add(aligned - addr);
            }
            current = current.wrapping_add(1);
        }
        ptr as *mut RuntimeAccount
    }

    /// Resolve a duplicate index to its original non-duplicate account. The
    /// index chain is strictly decreasing by the loader contract, so the loop
    /// is bounded by the transaction's u8 account-index domain.
    #[inline(always)]
    unsafe fn resolve_without_lookup(&self, index: u8) -> AccountView {
        let mut target = index;
        loop {
            let account = self.raw_account_at(target);
            let borrow_state = (*account).borrow_state;
            if target == 0 || borrow_state == NON_DUP_MARKER {
                return AccountView::new_unchecked(account);
            }
            target = borrow_state;
        }
    }

    /// Advance past one account record and return its `AccountView`.
    ///
    /// Handles both non-duplicated accounts (walks past the record
    /// header + data + padding) and duplicated accounts (reads the
    /// earlier view from `lookup`).
    ///
    /// Also writes the resolved view back into `lookup[consumed]` so
    /// future dup references resolve correctly, then increments
    /// `consumed`.
    ///
    /// # Safety
    ///
    /// Must not be called if `consumed` has already reached the
    /// transaction's total `num_accounts` — there's no trailing account
    /// record at that point. The caller (the derive-generated
    /// dispatcher or a user-level `remaining_accounts()` walk) is
    /// responsible for checking this upfront.
    #[inline(always)]
    pub unsafe fn next(&mut self) -> AccountView {
        let account: *mut RuntimeAccount = self.ptr as *mut RuntimeAccount;

        // Advance 8 bytes at the head of every slot: covers the
        // rent_epoch trailer for non-dup slots, or the full
        // (dup_marker + 7 bytes padding) body of dup slots. Pinocchio's
        // `read_account` applies the same "out-of-order" advance — it's
        // algebraically equivalent to adding the struct size + data +
        // padding + alignment at the end.
        self.ptr = self.ptr.add(core::mem::size_of::<u64>());

        // First account (consumed == 0) can never be a duplicate —
        // short-circuits the dup check for the first field.
        let borrow_state = (*account).borrow_state;
        let view = if self.consumed == 0 || borrow_state == NON_DUP_MARKER {
            // Non-dup: write data_len into the padding slot so
            // `AccountView::resize()` can enforce
            // MAX_PERMITTED_DATA_INCREASE later without another syscall.
            (*account).padding = u32::to_le_bytes((*account).data_len as u32);
            let data_len = (*account).data_len as usize;
            self.ptr = self.ptr.add(STATIC_ACCOUNT_DATA);
            self.ptr = self.ptr.add(data_len);
            // Align to the next 8-byte boundary. Use strict provenance APIs to
            // allow this to be tested under Miri.
            let addr = self.ptr.addr();
            let aligned = (addr + (BPF_ALIGN_OF_U128 - 1)) & !(BPF_ALIGN_OF_U128 - 1);
            self.ptr = self.ptr.add(aligned - addr);
            AccountView::new_unchecked(account)
        } else {
            // Duplicate: resolve through either the compact input scan or
            // the legacy lookup table. Safe because the runtime only emits
            // dup indices that are strictly less than the current `consumed`.
            // Lazy-materialize the bitvec on first dup so non-dup txs pay
            // zero initialization cost.
            let bv = self.duplicate.get_or_insert_with(AccountBitvec::default);
            bv.set(self.consumed);
            bv.set(borrow_state);
            if self.lookup.is_null() {
                self.resolve_without_lookup(borrow_state)
            } else {
                *self.lookup.add(borrow_state as usize)
            }
        };

        // Record this view so later dup references can resolve it.
        if !self.lookup.is_null() {
            *self.lookup.add(self.consumed as usize) = view;
        }
        self.consumed = self.consumed.wrapping_add(1);
        view
    }
}

/// A 256-bit bitvec used to store boolean information during account loading.
/// Does not derive `Copy` to avoid accidental large stack moves.
#[derive(Default, Clone)]
pub struct AccountBitvec {
    data: [u64; 4],
}

impl AccountBitvec {
    #[inline]
    pub fn get(&self, index: u8) -> bool {
        let index = index as usize;
        let arr_index = index / 64;
        let bit_index = index % 64;
        (self.data[arr_index] >> bit_index) & 1 == 1
    }

    #[inline]
    fn set(&mut self, index: u8) {
        let index = index as usize;
        let arr_index = index / 64;
        let bit_index = index % 64;
        self.data[arr_index] |= 1 << bit_index
    }

    /// `true` iff any bit set in `self` is also set in `mask`. Used by the
    /// dispatcher to fold every mutable-field dup check in a struct into a
    /// single 4-word AND+test instead of one `get()` per mut field.
    #[inline]
    pub fn intersects(&self, mask: &[u64; 4]) -> bool {
        (self.data[0] & mask[0])
            | (self.data[1] & mask[1])
            | (self.data[2] & mask[2])
            | (self.data[3] & mask[3])
            != 0
    }
}

/// OR `1 << bit` into `mask`. Used by the derive to build `TryAccounts::MUT_MASK`
/// at compile time from each direct mut field's offset.
pub const fn mut_mask_set_bit(mut mask: [u64; 4], bit: usize) -> [u64; 4] {
    mask[bit / 64] |= 1u64 << (bit % 64);
    mask
}

/// OR `other << shift` (as a 256-bit shift) into `mask`. Used by the derive
/// to fold a `Nested<U>` child's `MUT_MASK` into its parent's at the child's
/// account offset.
pub const fn mut_mask_or_shifted(mut mask: [u64; 4], other: [u64; 4], shift: usize) -> [u64; 4] {
    let word_shift = shift / 64;
    let bit_shift = shift % 64;
    let mut i = 0;
    while i < 4 {
        let src = other[i];
        let dst_lo = i + word_shift;
        if dst_lo < 4 {
            mask[dst_lo] |= src << bit_shift;
            if bit_shift != 0 && dst_lo + 1 < 4 {
                mask[dst_lo + 1] |= src >> (64 - bit_shift);
            }
        }
        i += 1;
    }
    mask
}

#[cfg(test)]
mod test {
    use super::*;

    // Exhaustively covers the four AccountBitvec invariants (default-is-empty,
    // set-then-get, non-interference, idempotence) over the full u8 index
    // domain. Kani adds no value here — the index fits in 256 values.
    //
    // `AccountCursor::next` itself isn't covered by a unit test: its body
    // reads through the raw input pointer written by the SBF loader.
    // The same logic is witnessed at runtime by
    // `lang-v2/tests/miri_cursor_walk.rs`, which uses the `SbfInputBuffer`
    // mock exported from `anchor_lang::testing`.
    #[test]
    fn test_bitvec() {
        let mut bv = AccountBitvec::default();
        for i in 0..=255u8 {
            assert!(!bv.get(i), "bit {i} set before any set() call");
        }
        for i in 0..=255u8 {
            bv.set(i);
            assert!(bv.get(i));
            // Idempotent: a second set() leaves the backing state unchanged.
            let before = bv.data;
            bv.set(i);
            assert!(bv.data == before, "set({i}) is not idempotent");
            // Non-interference: every previously-set bit is still set.
            for j in 0..=i {
                assert!(bv.get(j), "setting bit {i} cleared bit {j}");
            }
        }
    }
}
