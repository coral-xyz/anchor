extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

use anchor_syn::codegen::error as error_codegen;
use anchor_syn::parser::error as error_parser;
use anchor_syn::ErrorArgs;
use syn::{
    parse::{Parse, Result as ParseResult},
    parse_macro_input, Expr, Token,
};

/// Generates `Error` and `type Result<T> = Result<T, Error>` types to be
/// used as return types from Anchor instruction handlers. Importantly, the
/// attribute implements
/// [`From`](https://doc.rust-lang.org/std/convert/trait.From.html) on the
/// `ErrorCode` to support converting from the user defined error enum *into*
/// the generated `Error`.
///
/// # Example
///
/// ```ignore
/// use anchor_lang::prelude::*;
///
/// #[program]
/// mod errors {
///     use super::*;
///     pub fn hello(_ctx: Context<Hello>) -> Result<()> {
///         Err(MyError::Hello.into())
///     }
/// }
///
/// #[derive(Accounts)]
/// pub struct Hello {}
///
/// #[error]
/// pub enum MyError {
///     #[msg("This is an error message clients will automatically display")]
///     Hello,
/// }
/// ```
///
/// Note that we generate a new `Error` type so that we can return either the
/// user defined error enum *or* a
/// [`ProgramError`](../solana_program/enum.ProgramError.html), which is used
/// pervasively, throughout solana program crates. The generated `Error` type
/// should almost never be used directly, as the user defined error is
/// preferred. In the example above, `MyError::Hello.into()`.
///
/// # Msg
///
/// The `#[msg(..)]` attribute is inert, and is used only as a marker so that
/// parsers  and IDLs can map error codes to error messages.
#[proc_macro_attribute]
pub fn error_codes(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = match args.is_empty() {
        true => None,
        false => Some(parse_macro_input!(args as ErrorArgs)),
    };
    let mut error_enum = parse_macro_input!(input as syn::ItemEnum);
    let error = error_codegen::generate(error_parser::parse(&mut error_enum, args));
    proc_macro::TokenStream::from(error)
}

#[proc_macro]
pub fn error_without_origin(ts: proc_macro::TokenStream) -> TokenStream {
    let input = parse_macro_input!(ts as ErrorInput);
    let error_code = input.error_code;
    TokenStream::from(quote! {
        anchor_lang::error::Error::from(
            anchor_lang::error::AnchorError {
                error_name: #error_code.name(),
                error_code_number: #error_code.into(),
                error_msg: #error_code.to_string(),
                source: None,
                account_name: None
            }
        )
    })
}

#[proc_macro]
pub fn error(ts: proc_macro::TokenStream) -> TokenStream {
    let input = parse_macro_input!(ts as ErrorInput);
    let error_code = input.error_code;
    TokenStream::from(quote! {
        anchor_lang::error::Error::from(
            anchor_lang::error::AnchorError {
                error_name: #error_code.name(),
                error_code_number: #error_code.into(),
                error_msg: #error_code.to_string(),
                source: Some(anchor_lang::error::Source {
                    filename: file!(),
                    line: line!()
                }),
                account_name: None
            }
        )
    })
}

#[proc_macro]
pub fn error_with_account_name(ts: proc_macro::TokenStream) -> TokenStream {
    let input = parse_macro_input!(ts as ErrorWithAccountNameInput);
    let error_code = input.error_code;
    let account_name = input.account_name;
    TokenStream::from(quote! {
        anchor_lang::error::Error::from(
            anchor_lang::error::AnchorError {
                error_name: #error_code.name(),
                error_code_number: #error_code.into(),
                error_msg: #error_code.to_string(),
                source: None,
                account_name: Some(#account_name.to_string())
            }
        )
    })
}

struct ErrorInput {
    error_code: Expr,
}

impl Parse for ErrorInput {
    fn parse(stream: syn::parse::ParseStream) -> ParseResult<Self> {
        let error_code = stream.call(Expr::parse)?;
        Ok(Self { error_code })
    }
}

struct ErrorWithAccountNameInput {
    error_code: Expr,
    account_name: Expr,
}

impl Parse for ErrorWithAccountNameInput {
    fn parse(stream: syn::parse::ParseStream) -> ParseResult<Self> {
        let error_code = stream.call(Expr::parse)?;
        let _ = stream.parse::<Token!(,)>();
        let account_name = stream.call(Expr::parse)?;
        Ok(Self {
            error_code,
            account_name,
        })
    }
}
