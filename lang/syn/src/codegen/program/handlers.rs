use crate::codegen::program::common::*;
use crate::program_codegen::idl::idl_accounts_and_functions;
use crate::Program;
use heck::CamelCase;
use quote::{quote, ToTokens};

// Generate non-inlined wrappers for each instruction handler, since Solana's
// BPF max stack size can't handle reasonable sized dispatch trees without doing
// so.
pub fn generate(program: &Program) -> proc_macro2::TokenStream {
    let program_name = &program.name;
    // A constant token stream that stores the accounts and functions, required to live
    // inside the target program in order to get the program ID.
    let idl_accounts_and_functions = idl_accounts_and_functions();
    let non_inlined_idl: proc_macro2::TokenStream = {
        quote! {
            // Entry for all IDL related instructions. Use the "no-idl" feature
            // to eliminate this code, for example, if one wants to make the
            // IDL no longer mutable or if one doesn't want to store the IDL
            // on chain.
            #[inline(never)]
            #[cfg(not(feature = "no-idl"))]
            pub fn __idl_dispatch(program_id: &Pubkey, accounts: &[AccountInfo], idl_ix_data: &[u8]) -> anchor_lang::Result<()> {
                let mut accounts = accounts;
                let mut data: &[u8] = idl_ix_data;

                let ix = anchor_lang::idl::IdlInstruction::deserialize(&mut data)
                    .map_err(|_| anchor_lang::error::ErrorCode::InstructionDidNotDeserialize)?;

                match ix {
                    anchor_lang::idl::IdlInstruction::Create { data_len } => {
                        let mut bumps = std::collections::BTreeMap::new();
                        let mut reallocs = std::collections::BTreeSet::new();
                        let mut accounts =
                            IdlCreateAccounts::try_accounts(program_id, &mut accounts, &[], &mut bumps, &mut reallocs)?;
                        __idl_create_account(program_id, &mut accounts, data_len)?;
                        accounts.exit(program_id)?;
                    },
                    anchor_lang::idl::IdlInstruction::Resize { data_len } => {
                        let mut bumps = std::collections::BTreeMap::new();
                        let mut reallocs = std::collections::BTreeSet::new();
                        let mut accounts =
                            IdlResizeAccount::try_accounts(program_id, &mut accounts, &[], &mut bumps, &mut reallocs)?;
                        __idl_resize_account(program_id, &mut accounts, data_len)?;
                        accounts.exit(program_id)?;
                    },
                    anchor_lang::idl::IdlInstruction::Close => {
                        let mut bumps = std::collections::BTreeMap::new();
                        let mut reallocs = std::collections::BTreeSet::new();
                        let mut accounts =
                            IdlCloseAccount::try_accounts(program_id, &mut accounts, &[], &mut bumps, &mut reallocs)?;
                        __idl_close_account(program_id, &mut accounts)?;
                        accounts.exit(program_id)?;
                    },
                    anchor_lang::idl::IdlInstruction::CreateBuffer => {
                        let mut bumps = std::collections::BTreeMap::new();
                        let mut reallocs = std::collections::BTreeSet::new();
                        let mut accounts =
                            IdlCreateBuffer::try_accounts(program_id, &mut accounts, &[], &mut bumps, &mut reallocs)?;
                        __idl_create_buffer(program_id, &mut accounts)?;
                        accounts.exit(program_id)?;
                    },
                    anchor_lang::idl::IdlInstruction::Write { data } => {
                        let mut bumps = std::collections::BTreeMap::new();
                        let mut reallocs = std::collections::BTreeSet::new();
                        let mut accounts =
                            IdlAccounts::try_accounts(program_id, &mut accounts, &[], &mut bumps, &mut reallocs)?;
                        __idl_write(program_id, &mut accounts, data)?;
                        accounts.exit(program_id)?;
                    },
                    anchor_lang::idl::IdlInstruction::SetAuthority { new_authority } => {
                        let mut bumps = std::collections::BTreeMap::new();
                        let mut reallocs = std::collections::BTreeSet::new();
                        let mut accounts =
                            IdlAccounts::try_accounts(program_id, &mut accounts, &[], &mut bumps, &mut reallocs)?;
                        __idl_set_authority(program_id, &mut accounts, new_authority)?;
                        accounts.exit(program_id)?;
                    },
                    anchor_lang::idl::IdlInstruction::SetBuffer => {
                        let mut bumps = std::collections::BTreeMap::new();
                        let mut reallocs = std::collections::BTreeSet::new();
                        let mut accounts =
                            IdlSetBuffer::try_accounts(program_id, &mut accounts, &[], &mut bumps, &mut reallocs)?;
                        __idl_set_buffer(program_id, &mut accounts)?;
                        accounts.exit(program_id)?;
                    },
                }
                Ok(())
            }

        }
    };

    let non_inlined_handlers: Vec<proc_macro2::TokenStream> = program
        .ixs
        .iter()
        .map(|ix| {
            let ix_arg_names: Vec<&syn::Ident> = ix.args.iter().map(|arg| &arg.name).collect();
            let ix_name = generate_ix_variant_name(ix.raw_method.sig.ident.to_string());
            let ix_method_name = &ix.raw_method.sig.ident;
            let anchor = &ix.anchor_ident;
            let variant_arm = generate_ix_variant(ix.raw_method.sig.ident.to_string(), &ix.args);
            let ix_name_log = format!("Instruction: {ix_name}");
            let ret_type = &ix.returns.ty.to_token_stream();
            let maybe_set_return_data = match ret_type.to_string().as_str() {
                "()" => quote! {},
                _ => quote! {
                    anchor_lang::solana_program::program::set_return_data(&result.try_to_vec().unwrap());
                },
            };
            quote! {
                #[inline(never)]
                pub fn #ix_method_name(
                    program_id: &Pubkey,
                    accounts: &[AccountInfo],
                    ix_data: &[u8],
                ) -> anchor_lang::Result<()> {
                    #[cfg(not(feature = "no-log-ix-name"))]
                    anchor_lang::prelude::msg!(#ix_name_log);

                    // Deserialize data.
                    let ix = instruction::#ix_name::deserialize(&mut &ix_data[..])
                        .map_err(|_| anchor_lang::error::ErrorCode::InstructionDidNotDeserialize)?;
                    let instruction::#variant_arm = ix;

                    // Bump collector.
                    let mut __bumps = std::collections::BTreeMap::new();

                    let mut __reallocs = std::collections::BTreeSet::new();

                    // Deserialize accounts.
                    let mut remaining_accounts: &[AccountInfo] = accounts;
                    let mut accounts = #anchor::try_accounts(
                        program_id,
                        &mut remaining_accounts,
                        ix_data,
                        &mut __bumps,
                        &mut __reallocs,
                    )?;

                    // Invoke user defined handler.
                    let result = #program_name::#ix_method_name(
                        anchor_lang::context::Context::new(
                            program_id,
                            &mut accounts,
                            remaining_accounts,
                            __bumps,
                        ),
                        #(#ix_arg_names),*
                    )?;

                    // Maybe set Solana return data.
                    #maybe_set_return_data

                    // Exit routine.
                    accounts.exit(program_id)
                }
            }
        })
        .collect();

    quote! {
        /// Create a private module to not clutter the program's namespace.
        /// Defines an entrypoint for each individual instruction handler
        /// wrapper.
        mod __private {
            use super::*;
            /// __idl mod defines handlers for injected Anchor IDL instructions.
            pub mod __idl {
                use super::*;

                #non_inlined_idl
                #idl_accounts_and_functions
            }



            /// __global mod defines wrapped handlers for global instructions.
            pub mod __global {
                use super::*;

                #(#non_inlined_handlers)*
            }
        }
    }
}

fn generate_ix_variant_name(name: String) -> proc_macro2::TokenStream {
    let n = name.to_camel_case();
    n.parse().unwrap()
}
