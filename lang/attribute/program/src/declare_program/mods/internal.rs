use anchor_lang_idl::types::{Idl, IdlInstructionAccountItem};
use anchor_syn::{
    codegen::accounts::{__client_accounts, __cpi_client_accounts},
    parser::accounts,
    AccountsStruct,
};
use heck::CamelCase;
use quote::{format_ident, quote};

use super::common::{convert_idl_type_to_syn_type, gen_discriminator, get_canonical_program_id};

pub fn gen_internal_mod(idl: &Idl) -> proc_macro2::TokenStream {
    let internal_args_mod = gen_internal_args_mod(idl);
    let internal_accounts_mod = gen_internal_accounts(idl);

    quote! {
        #[doc(hidden)]
        mod internal {
            use super::*;

            #internal_args_mod
            #internal_accounts_mod
        }
    }
}

fn gen_internal_args_mod(idl: &Idl) -> proc_macro2::TokenStream {
    let ixs = idl.instructions.iter().map(|ix| {
        let ix_struct_name = format_ident!("{}", ix.name.to_camel_case());

        let fields = ix.args.iter().map(|arg| {
            let name = format_ident!("{}", arg.name);
            let ty = convert_idl_type_to_syn_type(&arg.ty);
            quote! { pub #name: #ty }
        });

        let ix_struct = if ix.args.is_empty() {
            quote! {
                pub struct #ix_struct_name;
            }
        } else {
            quote! {
                pub struct #ix_struct_name {
                    #(#fields),*
                }
            }
        };

        let impl_discriminator = if ix.discriminator.len() == 8 {
            let discriminator = gen_discriminator(&ix.discriminator);
            quote! {
                impl anchor_lang::Discriminator for #ix_struct_name {
                    const DISCRIMINATOR: [u8; 8] = #discriminator;
                }
            }
        } else {
            quote! {}
        };

        let impl_ix_data = quote! {
            impl anchor_lang::InstructionData for #ix_struct_name {}
        };

        let program_id = get_canonical_program_id();
        let impl_owner = quote! {
            impl anchor_lang::Owner for #ix_struct_name {
                fn owner() -> Pubkey {
                    #program_id
                }
            }
        };

        quote! {
            /// Instruction argument
            #[derive(AnchorSerialize, AnchorDeserialize)]
            #ix_struct

            #impl_discriminator
            #impl_ix_data
            #impl_owner
        }
    });

    quote! {
        /// An Anchor generated module containing the program's set of instructions, where each
        /// method handler in the `#[program]` mod is associated with a struct defining the input
        /// arguments to the method. These should be used directly, when one wants to serialize
        /// Anchor instruction data, for example, when specifying instructions instructions on a
        /// client.
        pub mod args {
            use super::*;

            #(#ixs)*
        }
    }
}

fn gen_internal_accounts(idl: &Idl) -> proc_macro2::TokenStream {
    let cpi_accounts = gen_internal_accounts_common(idl, __cpi_client_accounts::generate);
    let client_accounts = gen_internal_accounts_common(idl, __client_accounts::generate);

    quote! {
        #cpi_accounts
        #client_accounts
    }
}

fn gen_internal_accounts_common(
    idl: &Idl,
    gen_accounts: impl Fn(&AccountsStruct, proc_macro2::TokenStream) -> proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let accounts = idl
        .instructions
        .iter()
        .map(|ix| {
            let ident = format_ident!("{}", ix.name.to_camel_case());
            let generics = if ix.accounts.is_empty() {
                quote!()
            } else {
                quote!(<'info>)
            };
            let accounts = ix.accounts.iter().map(|acc| match acc {
                IdlInstructionAccountItem::Single(acc) => {
                    let name = format_ident!("{}", acc.name);

                    let attrs = {
                        let signer = acc.signer.then(|| quote!(signer)).unwrap_or_default();
                        let mt = acc.writable.then(|| quote!(mut)).unwrap_or_default();
                        if signer.is_empty() {
                            mt
                        } else if mt.is_empty() {
                            signer
                        } else {
                            quote! { #signer, #mt }
                        }
                    };

                    let acc_expr = acc
                        .optional
                        .then(|| quote! { Option<AccountInfo #generics> })
                        .unwrap_or_else(|| quote! { AccountInfo #generics });

                    quote! {
                        #[account(#attrs)]
                        pub #name: #acc_expr
                    }
                }
                IdlInstructionAccountItem::Composite(accs) => {
                    let name = format_ident!("{}", accs.name);
                    let ty_name = idl
                        .instructions
                        .iter()
                        .find(|ix| ix.accounts == accs.accounts)
                        .map(|ix| format_ident!("{}", ix.name.to_camel_case()))
                        .expect("Instruction must exist");

                    quote! {
                        pub #name: #ty_name #generics
                    }
                }
            });

            quote! {
                #[derive(Accounts)]
                pub struct #ident #generics {
                    #(#accounts,)*
                }
            }
        })
        .map(|accs_struct| {
            let accs_struct = syn::parse2(accs_struct).expect("Failed to parse as syn::ItemStruct");
            let accs_struct =
                accounts::parse(&accs_struct).expect("Failed to parse accounts struct");
            gen_accounts(&accs_struct, get_canonical_program_id())
        });

    quote! { #(#accounts)* }
}
