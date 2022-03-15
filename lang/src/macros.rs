/// Ensures a condition is true, otherwise returns with the given error.
/// Use this with or without a custom error type.
///
/// # Example
/// ```ignore
/// // Instruction function
/// pub fn set_data(ctx: Context<SetData>, data: u64) -> Result<()> {
///     require!(ctx.accounts.data.mutation_allowed, MyError::MutationForbidden);
///     ctx.accounts.data.data = data;
///     Ok(())
/// }
///
/// // An enum for custom error codes
/// #[error_code]
/// pub enum MyError {
///     MutationForbidden
/// }
///
/// // An account definition
/// #[account]
/// #[derive(Default)]
/// pub struct MyData {
///     mutation_allowed: bool,
///     data: u64
/// }
///
/// // An account validation struct
/// #[derive(Accounts)]
/// pub struct SetData<'info> {
///     #[account(mut)]
///     pub data: Account<'info, MyData>
/// }
/// ```
#[macro_export]
macro_rules! require {
    ($invariant:expr, $error:tt $(,)?) => {
        if !($invariant) {
            return Err(anchor_lang::anchor_attribute_error::error!(
                crate::ErrorCode::$error
            ));
        }
    };
    ($invariant:expr, $error:expr $(,)?) => {
        if !($invariant) {
            return Err(anchor_lang::anchor_attribute_error::error!($error));
        }
    };
}

/// Ensures two NON-PUBKEY values are equal.
///
/// Use [require_keys_eq](crate::prelude::require_keys_eq)
/// to compare two pubkeys.
///
/// Can be used with or without a custom error code.
///
/// # Example
/// ```rust,ignore
/// pub fn set_data(ctx: Context<SetData>, data: u64) -> Result<()> {
///     require_eq!(ctx.accounts.data.data, 0);
///     ctx.accounts.data.data = data;
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! require_eq {
    ($value1: expr, $value2: expr, $error_code:expr $(,)?) => {
        if $value1 != $value2 {
            return Err(error!($error_code).with_values(($value1, $value2)));
        }
    };
    ($value1: expr, $value2: expr $(,)?) => {
        if $value1 != $value2 {
            return Err(error!(anchor_lang::error::ErrorCode::RequireEqViolated)
                .with_values(($value1, $value2)));
        }
    };
}

/// Ensures two NON-PUBKEY values are not equal.
///
/// Use [require_keys_neq](crate::prelude::require_keys_neq)
/// to compare two pubkeys.
///
/// Can be used with or without a custom error code.
///
/// # Example
/// ```rust,ignore
/// pub fn set_data(ctx: Context<SetData>, data: u64) -> Result<()> {
///     require_neq!(ctx.accounts.data.data, 0);
///     ctx.accounts.data.data = data;
///     Ok(());
/// }
/// ```
#[macro_export]
macro_rules! require_neq {
    ($value1: expr, $value2: expr, $error_code: expr $(,)?) => {
        if $value1 == $value2 {
            return Err(error!($error_code).with_values(($value1, $value2)));
        }
    };
    ($value1: expr, $value2: expr $(,)?) => {
        if $value1 == $value2 {
            return Err(error!(anchor_lang::error::ErrorCode::RequireEqViolated)
                .with_values(($value1, $value2)));
        }
    };
}

/// Ensures two pubkeys values are equal.
///
/// Use [require_eq](crate::prelude::require_eq)
/// to compare two non-pubkey values.
///
/// Can be used with or without a custom error code.
///
/// # Example
/// ```rust,ignore
/// pub fn set_data(ctx: Context<SetData>, data: u64) -> Result<()> {
///     require_keys_eq!(ctx.accounts.data.authority.key(), ctx.accounts.authority.key());
///     ctx.accounts.data.data = data;
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! require_keys_eq {
    ($value1: expr, $value2: expr, $error_code:expr $(,)?) => {
        if $value1 != $value2 {
            return Err(error!($error_code).with_pubkeys(($value1, $value2)));
        }
    };
    ($value1: expr, $value2: expr $(,)?) => {
        if $value1 != $value2 {
            return Err(error!(anchor_lang::error::ErrorCode::RequireKeysEqViolated)
                .with_pubkeys(($value1, $value2)));
        }
    };
}

/// Ensures two pubkeys values are not equal.
///
/// Use [require_neq](crate::prelude::require_neq)
/// to compare two non-pubkey values.
///
/// Can be used with or without a custom error code.
///
/// # Example
/// ```rust,ignore
/// pub fn set_data(ctx: Context<SetData>, data: u64) -> Result<()> {
///     require_keys_neq!(ctx.accounts.data.authority.key(), ctx.accounts.other.key());
///     ctx.accounts.data.data = data;
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! require_keys_neq {
    ($value1: expr, $value2: expr, $error_code: expr $(,)?) => {
        if $value1 == $value2 {
            return Err(error!($error_code).with_pubkeys(($value1, $value2)));
        }
    };
    ($value1: expr, $value2: expr $(,)?) => {
        if $value1 == $value2 {
            return Err(error!(anchor_lang::error::ErrorCode::RequireKeysEqViolated)
                .with_pubkeys(($value1, $value2)));
        }
    };
}

/// Ensures the first NON-PUBKEY value is greater than the second
/// NON-PUBKEY value.
///
/// Can be used with or without a custom error code.
///
/// # Example
/// ```rust,ignore
/// pub fn set_data(ctx: Context<SetData>, data: u64) -> Result<()> {
///     require_gt!(ctx.accounts.data.data, 0);
///     ctx.accounts.data.data = data;
///     Ok(());
/// }
/// ```
#[macro_export]
macro_rules! require_gt {
    ($value1: expr, $value2: expr, $error_code: expr $(,)?) => {
        if $value1 <= $value2 {
            return Err(error!($error_code).with_values(($value1, $value2)));
        }
    };
    ($value1: expr, $value2: expr $(,)?) => {
        if $value1 <= $value2 {
            return Err(error!(anchor_lang::error::ErrorCode::RequireEqViolated)
                .with_values(($value1, $value2)));
        }
    };
}

/// Ensures the first NON-PUBKEY value is greater than or equal
/// to the second NON-PUBKEY value.
///
/// Can be used with or without a custom error code.
///
/// # Example
/// ```rust,ignore
/// pub fn set_data(ctx: Context<SetData>, data: u64) -> Result<()> {
///     require_gte!(ctx.accounts.data.data, 1);
///     ctx.accounts.data.data = data;
///     Ok(());
/// }
/// ```
#[macro_export]
macro_rules! require_gte {
    ($value1: expr, $value2: expr, $error_code: expr $(,)?) => {
        if $value1 < $value2 {
            return Err(error!($error_code).with_values(($value1, $value2)));
        }
    };
    ($value1: expr, $value2: expr $(,)?) => {
        if $value1 < $value2 {
            return Err(error!(anchor_lang::error::ErrorCode::RequireEqViolated)
                .with_values(($value1, $value2)));
        }
    };
}

/// Returns with the given error.
/// Use this with a custom error type.
///
/// # Example
/// ```ignore
/// // Instruction function
/// pub fn example(ctx: Context<Example>) -> Result<()> {
///     err!(MyError::SomeError)
/// }
///
/// // An enum for custom error codes
/// #[error_code]
/// pub enum MyError {
///     SomeError
/// }
/// ```
#[macro_export]
macro_rules! err {
    ($error:tt $(,)?) => {
        Err(anchor_lang::anchor_attribute_error::error!(
            crate::ErrorCode::$error
        ))
    };
    ($error:expr $(,)?) => {
        Err(anchor_lang::anchor_attribute_error::error!($error))
    };
}

/// Creates a [`Source`](crate::error::Source)
#[macro_export]
macro_rules! source {
    () => {
        anchor_lang::error::Source {
            filename: file!(),
            line: line!(),
        }
    };
}
