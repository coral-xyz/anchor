use crate::Error;
use quote::quote;

pub fn generate(error: Error) -> proc_macro2::TokenStream {
    let error_enum = &error.raw_enum;
    let enum_name = &error.ident;
    // Each arm of the `match` statement for implementing `std::fmt::Display`
    // on the user defined error code.
    let variant_dispatch: Vec<proc_macro2::TokenStream> = error
        .raw_enum
        .variants
        .iter()
        .enumerate()
        .map(|(idx, variant)| {
            let ident = &variant.ident;
            let error_code = &error.codes[idx];
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
            quote! {
                #enum_name::#ident => #msg
            }
        })
        .collect();

    let offset = match error.args {
        None => quote! { anchor_lang::error::ERROR_CODE_OFFSET},
        Some(args) => {
            let offset = &args.offset;
            quote! { #offset }
        }
    };

    quote! {
        #[derive(std::fmt::Debug, Clone, Copy)]
        #[repr(u32)]
        #error_enum

        impl From<#enum_name> for u32 {
            fn from(e: #enum_name) -> u32 {
                e as u32 + #offset
            }
        }

        impl std::fmt::Display for #enum_name {
            fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                match self {
                    #(#variant_dispatch),*
                }
            }
        }
    }
}
