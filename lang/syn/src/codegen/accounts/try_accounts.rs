use crate::codegen::accounts::{constraints, generics};
use crate::{AccountField, AccountsStruct, Field, SysvarTy, Ty};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

// Generates the `Accounts` trait implementation.
pub fn generate(accs: &AccountsStruct) -> proc_macro2::TokenStream {
    let name = &accs.ident;
    let (combined_generics, trait_generics, strct_generics) = generics(accs);

    // Deserialization for each field
    let deser_fields: Vec<proc_macro2::TokenStream> = accs
        .fields
        .iter()
        .map(|af: &AccountField| {
            match af {
                AccountField::CompositeField(s) => {
                    let name = &s.ident;
                    let ty = &s.raw_field.ty;
                    quote! {
                        #[cfg(feature = "anchor-debug")]
                        ::solana_program::log::sol_log(stringify!(#name));
                        let #name: #ty = anchor_lang::Accounts::try_accounts(program_id, accounts, ix_data)?;
                    }
                }
                AccountField::Field(f) => {
                    // Associated fields are *first* deserialized into
                    // AccountInfos, and then later deserialized into
                    // ProgramAccounts in the "constraint check" phase.
                    if is_pda_init(af) {
                        let name = &f.ident;
                        quote!{
                            let #name = &accounts[0];
                            *accounts = &accounts[1..];
                        }
                    } else {
                        let name = typed_ident(f);
                        match f.constraints.is_init() {
                            false => quote! {
                                #[cfg(feature = "anchor-debug")]
                                ::solana_program::log::sol_log(stringify!(#name));
                                let #name = anchor_lang::Accounts::try_accounts(program_id, accounts, ix_data)?;
                            },
                            true => quote! {
                                #[cfg(feature = "anchor-debug")]
                                ::solana_program::log::sol_log(stringify!(#name));
                                let #name = anchor_lang::AccountsInit::try_accounts_init(program_id, accounts)?;
                            },
                        }
                    }
                }
            }
        })
        .collect();

    let constraints = generate_constraints(accs);
    let accounts_instance = generate_accounts_instance(accs);

    let ix_de = match &accs.instruction_api {
        None => quote! {},
        Some(ix_api) => {
            let strct_inner = &ix_api;
            let field_names: Vec<proc_macro2::TokenStream> = ix_api
                .iter()
                .map(|expr: &Expr| match expr {
                    Expr::Type(expr_type) => {
                        let field = &expr_type.expr;
                        quote! {
                            #field
                        }
                    }
                    _ => panic!("Invalid instruction declaration"),
                })
                .collect();
            quote! {
                let mut ix_data = ix_data;
                #[derive(anchor_lang::AnchorSerialize, anchor_lang::AnchorDeserialize)]
                struct __Args {
                    #strct_inner
                }
                let __Args {
                    #(#field_names),*
                } = __Args::deserialize(&mut ix_data)
                    .map_err(|_| anchor_lang::__private::ErrorCode::InstructionDidNotDeserialize)?;
            }
        }
    };

    quote! {
        impl#combined_generics anchor_lang::Accounts#trait_generics for #name#strct_generics {
            #[inline(never)]
            fn try_accounts(
                program_id: &anchor_lang::solana_program::pubkey::Pubkey,
                accounts: &mut &[anchor_lang::solana_program::account_info::AccountInfo<'info>],
                ix_data: &[u8],
            ) -> std::result::Result<Self, anchor_lang::solana_program::program_error::ProgramError> {
                // Deserialize instruction, if declared.
                #ix_de
                // Deserialize each account.
                #(#deser_fields)*
                // Execute accounts constraints.
                #constraints
                // Success. Return the validated accounts.
                Ok(#accounts_instance)
            }
        }
    }
}

// Returns true if the given AccountField has an associated init constraint.
fn is_pda_init(af: &AccountField) -> bool {
    match af {
        AccountField::CompositeField(_s) => false,
        AccountField::Field(f) => {
            f.constraints
                .associated
                .as_ref()
                .map(|f| f.is_init)
                .unwrap_or(false)
                || f.constraints
                    .seeds
                    .as_ref()
                    .map(|f| f.is_init)
                    .unwrap_or(false)
        }
    }
}

fn typed_ident(field: &Field) -> TokenStream {
    let name = &field.ident;

    let ty = match &field.ty {
        Ty::AccountInfo => quote! { AccountInfo },
        Ty::ProgramState(ty) => {
            let account = &ty.account_ident;
            quote! {
                ProgramState<#account>
            }
        }
        Ty::CpiState(ty) => {
            let account = &ty.account_ident;
            quote! {
                CpiState<#account>
            }
        }
        Ty::ProgramAccount(ty) => {
            let account = &ty.account_ident;
            quote! {
                ProgramAccount<#account>
            }
        }
        Ty::Loader(ty) => {
            let account = &ty.account_ident;
            quote! {
                Loader<#account>
            }
        }
        Ty::CpiAccount(ty) => {
            let account = &ty.account_ident;
            quote! {
                CpiAccount<#account>
            }
        }
        Ty::Sysvar(ty) => {
            let account = match ty {
                SysvarTy::Clock => quote! {Clock},
                SysvarTy::Rent => quote! {Rent},
                SysvarTy::EpochSchedule => quote! {EpochSchedule},
                SysvarTy::Fees => quote! {Fees},
                SysvarTy::RecentBlockhashes => quote! {RecentBlockhashes},
                SysvarTy::SlotHashes => quote! {SlotHashes},
                SysvarTy::SlotHistory => quote! {SlotHistory},
                SysvarTy::StakeHistory => quote! {StakeHistory},
                SysvarTy::Instructions => quote! {Instructions},
                SysvarTy::Rewards => quote! {Rewards},
            };
            quote! {
                Sysvar<#account>
            }
        }
    };

    quote! {
        #name: #ty
    }
}

pub fn generate_constraints(accs: &AccountsStruct) -> proc_macro2::TokenStream {
    // All fields without an `#[account(associated)]` attribute.
    let non_associated_fields: Vec<&AccountField> =
        accs.fields.iter().filter(|af| !is_pda_init(af)).collect();

    // Deserialization for each *associated* field. This must be after
    // the inital extraction from the accounts slice and before access_checks.
    let init_associated_fields: Vec<proc_macro2::TokenStream> = accs
        .fields
        .iter()
        .filter_map(|af| match af {
            AccountField::CompositeField(_s) => None,
            AccountField::Field(f) => match is_pda_init(af) {
                false => None,
                true => Some(f),
            },
        })
        .map(constraints::generate)
        .collect();

    // Constraint checks for each account fields.
    let access_checks: Vec<proc_macro2::TokenStream> = non_associated_fields
        .iter()
        .map(|af: &&AccountField| match af {
            AccountField::Field(f) => constraints::generate(f),
            AccountField::CompositeField(s) => constraints::generate_composite(s),
        })
        .collect();

    quote! {
        #(#init_associated_fields)*
        #(#access_checks)*
    }
}

pub fn generate_accounts_instance(accs: &AccountsStruct) -> proc_macro2::TokenStream {
    let name = &accs.ident;
    // Each field in the final deserialized accounts struct.
    let return_tys: Vec<proc_macro2::TokenStream> = accs
        .fields
        .iter()
        .map(|f: &AccountField| {
            let name = match f {
                AccountField::CompositeField(s) => &s.ident,
                AccountField::Field(f) => &f.ident,
            };
            quote! {
                #name
            }
        })
        .collect();

    quote! {
        #name {
            #(#return_tys),*
        }
    }
}
