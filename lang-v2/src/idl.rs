#[cfg(target_os = "solana")]
use crate::FixedVec;

#[cfg(not(target_os = "solana"))]
pub type IdlEntries = std::vec::Vec<&'static str>;
#[cfg(target_os = "solana")]
pub type IdlEntries = FixedVec<&'static str, { crate::MAX_ACCOUNT_METAS }>;

/// Compile-time IDL metadata hooks. The registration surface is bounded so
/// the runtime crate does not require a heap even when IDL support is disabled.
#[diagnostic::on_unimplemented(
    message = "`{Self}` has no IDL type information",
    note = "plain structs used as instruction arguments or nested fields need `#[derive(IdlType)]`; account and event types get this automatically from `#[account]` / `#[event]`"
)]
pub trait IdlAccountType {
    const __IDL_ACCOUNT_ENTRY: Option<&'static str> = None;
    const __IDL_TYPE_DEF: Option<&'static str> = None;
    const __IDL_IS_SIGNER: bool = false;
    const __IDL_ADDRESS: Option<&'static str> = None;

    fn __idl_account_entry() -> Option<&'static str> {
        Self::__IDL_ACCOUNT_ENTRY
    }

    fn __idl_type_def() -> Option<&'static str> {
        Self::__IDL_TYPE_DEF
    }

    fn __register_idl_deps(_accounts: &mut IdlEntries, _types: &mut IdlEntries) {}
}

macro_rules! impl_leaf_idl_types {
    ($($ty:ty),* $(,)?) => {
        $(impl IdlAccountType for $ty {})*
    };
}

impl_leaf_idl_types!(
    (), bool, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64,
    crate::Address,
);

// Host/client IDL bindings may still use owned standard-library collections.
// Keep these impls outside the SBF target so the on-chain closure has no
// `std`/`alloc` dependency while `declare_program!` remains source-compatible
// for host consumers that do not enable the IDL build feature.
#[cfg(not(target_os = "solana"))]
impl IdlAccountType for std::string::String {}

#[cfg(not(target_os = "solana"))]
impl<T: IdlAccountType> IdlAccountType for std::vec::Vec<T> {
    fn __register_idl_deps(accounts: &mut IdlEntries, types: &mut IdlEntries) {
        T::__register_idl_deps(accounts, types);
    }
}

#[cfg(not(target_os = "solana"))]
impl<K: IdlAccountType, V: IdlAccountType> IdlAccountType
    for std::collections::BTreeMap<K, V>
{
    fn __register_idl_deps(accounts: &mut IdlEntries, types: &mut IdlEntries) {
        K::__register_idl_deps(accounts, types);
        V::__register_idl_deps(accounts, types);
    }
}

#[cfg(not(target_os = "solana"))]
impl<T: IdlAccountType> IdlAccountType for std::collections::BTreeSet<T> {
    fn __register_idl_deps(accounts: &mut IdlEntries, types: &mut IdlEntries) {
        T::__register_idl_deps(accounts, types);
    }
}

macro_rules! impl_idl_account_type_tuple {
    ($($ty:ident),+ $(,)?) => {
        #[doc(hidden)]
        impl<$($ty: IdlAccountType),+> IdlAccountType for ($($ty,)+) {
            fn __register_idl_deps(accounts: &mut IdlEntries, types: &mut IdlEntries) {
                $( $ty::__register_idl_deps(accounts, types); )+
            }
        }
    };
}

impl_idl_account_type_tuple!(A, B);
impl_idl_account_type_tuple!(A, B, C);
impl_idl_account_type_tuple!(A, B, C, D);
impl_idl_account_type_tuple!(A, B, C, D, E);
impl_idl_account_type_tuple!(A, B, C, D, E, F);
impl_idl_account_type_tuple!(A, B, C, D, E, F, G);
impl_idl_account_type_tuple!(A, B, C, D, E, F, G, H);
impl_idl_account_type_tuple!(A, B, C, D, E, F, G, H, I);
impl_idl_account_type_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_idl_account_type_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_idl_account_type_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_idl_account_type_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_idl_account_type_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_idl_account_type_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_idl_account_type_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

impl<T: IdlAccountType + ?Sized> IdlAccountType for &T {
    fn __register_idl_deps(accounts: &mut IdlEntries, types: &mut IdlEntries) {
        T::__register_idl_deps(accounts, types);
    }
}
impl<T: IdlAccountType> IdlAccountType for &mut T {
    fn __register_idl_deps(accounts: &mut IdlEntries, types: &mut IdlEntries) {
        T::__register_idl_deps(accounts, types);
    }
}
impl<T: IdlAccountType> IdlAccountType for Option<T> {
    fn __register_idl_deps(accounts: &mut IdlEntries, types: &mut IdlEntries) {
        T::__register_idl_deps(accounts, types);
    }
}
impl<T: IdlAccountType, const N: usize> IdlAccountType for [T; N] {
    fn __register_idl_deps(accounts: &mut IdlEntries, types: &mut IdlEntries) {
        T::__register_idl_deps(accounts, types);
    }
}

impl<T: IdlAccountType, const N: usize> IdlAccountType for crate::FixedVec<T, N> {
    fn __register_idl_deps(accounts: &mut IdlEntries, types: &mut IdlEntries) {
        T::__register_idl_deps(accounts, types);
    }
}

impl<T: IdlAccountType> IdlAccountType for [T] {
    fn __register_idl_deps(accounts: &mut IdlEntries, types: &mut IdlEntries) {
        T::__register_idl_deps(accounts, types);
    }
}
