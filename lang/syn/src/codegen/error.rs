use crate::Error;
use quote::quote;

pub fn generate(error: Error) -> proc_macro2::TokenStream {
    let error_enum = &error.raw_enum;
    let enum_name = &error.ident;
    // Each arm of the `match` statement for implementing `std::fmt::Display`
    // on the user defined error code.
    let (variant_dispatch, code_dispatch): (
        Vec<proc_macro2::TokenStream>,
        Vec<proc_macro2::TokenStream>,
    ) = error
        .raw_enum
        .variants
        .iter()
        .enumerate()
        .map(|(idx, variant)| {
            let ident = &variant.ident;
            let error_code = &error.codes[idx];
            let error_code_id = error_code.id;
            let msg = match &error_code.msg {
                None => {
                    quote! {
                        <Self as std::fmt::Debug>::fmt(self, fmt)
                    }
                }
                Some(msg) => {
                    quote! {
                        write!(fmt, #msg)
                    }
                }
            };
            let variant_dispatch = quote! {
                #enum_name::#ident => #msg
            };
            let code_dispatch = quote! {
                #error_code_id => Some(format!("{}", #enum_name::#ident))
            };
            (variant_dispatch, code_dispatch)
        })
        .clone()
        .unzip();

    let offset = match error.args {
        None => quote! { anchor_lang::__private::ERROR_CODE_OFFSET},
        Some(args) => {
            let offset = &args.offset;
            quote! { #offset }
        }
    };

    quote! {
        /// Anchor generated Result to be used as the return type for the
        /// program.
        pub type Result<T> = std::result::Result<T, Error>;

        pub fn __pretty_print_error_code(code: u32) -> Option<String> {
            match code - #offset {
                #(#code_dispatch),*
                , _ => None
            }
        }

        /// Anchor generated error allowing one to easily return a
        /// `ProgramError` or a custom, user defined error code by utilizing
        /// its `From` implementation.
        #[doc(hidden)]
        #[derive(thiserror::Error, Clone, Debug)]
        pub enum Error {
            #[error(transparent)]
            ProgramError(#[from] anchor_lang::solana_program::program_error::ProgramError),
            #[error(transparent)]
            ErrorCode(#[from] #enum_name),
        }

        #[derive(std::fmt::Debug, Clone, Copy)]
        #[repr(u32)]
        #error_enum

        impl std::fmt::Display for #enum_name {
            fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                match self {
                    #(#variant_dispatch),*
                }
            }
        }

        impl std::error::Error for #enum_name {}

        impl std::convert::From<Error> for anchor_lang::solana_program::program_error::ProgramError {
            fn from(e: Error) -> anchor_lang::solana_program::program_error::ProgramError {
                match e {
                    Error::ProgramError(e) => e,
                    Error::ErrorCode(c) => anchor_lang::solana_program::program_error::ProgramError::Custom(c as u32 + #offset),
                }
            }
        }

        impl std::convert::From<#enum_name> for anchor_lang::solana_program::program_error::ProgramError {
            fn from(e: #enum_name) -> anchor_lang::solana_program::program_error::ProgramError {
                let err: Error = e.into();
                err.into()
            }
        }
    }
}
