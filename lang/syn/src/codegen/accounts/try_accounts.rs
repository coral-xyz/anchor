use crate::codegen::accounts::{constraints, generics, ParsedGenerics};
use crate::{AccountField, AccountsStruct};
use quote::quote; 
use syn::Expr;

// Generates the `Accounts` trait implementation.
pub fn generate(accs: &AccountsStruct) -> proc_macro2::TokenStream {
    let name = &accs.ident;
    let ParsedGenerics {
        combined_generics,
        trait_generics,
        struct_generics,
        where_clause,
    } = generics(accs);

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
                        let #name: #ty = anchor_lang::Accounts::try_accounts(program_id, accounts, ix_data, __bumps)?;
                    }
                }
                AccountField::Field(f) => {
                    // `init` and `zero` acccounts are special cased as they are
                    // deserialized by constraints. Here, we just take out the
                    // AccountInfo for later use at constraint validation time.
                    if is_init(af) || f.constraints.zeroed.is_some() {
                        let name = &f.ident;
                        quote!{
                            if accounts.is_empty() {
                                return Err(anchor_lang::error::ErrorCode::AccountNotEnoughKeys.into());
                            }
                            let #name = &accounts[0];
                            *accounts = &accounts[1..];
                        }
                    } else {
                        let name = f.ident.to_string();
                        let typed_name = f.typed_ident();
                        quote! {
                            #[cfg(feature = "anchor-debug")]
                            ::solana_program::log::sol_log(stringify!(#typed_name));
                            let #typed_name = anchor_lang::Accounts::try_accounts(program_id, accounts, ix_data, __bumps)
                                .map_err(|e| e.with_account_name(#name))?;
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
                    .map_err(|_| anchor_lang::error::ErrorCode::InstructionDidNotDeserialize)?;
            }
        }
    };

    let error_handling = match &accs.bot_tax_api {
        None => quote! {},
        Some(bot_tax_api) => {
            let base_fee = &bot_tax_api.base_fee;
            let payer = &bot_tax_api.payer;
            let pay_to = &bot_tax_api.pay_to;
            let errors: Vec<proc_macro2::TokenStream> = bot_tax_api.error_codes
                .iter()
                .map(|err| quote!{
                   #err
                }) 
                .collect();

            // I think issue is that Ok(()) is wrong return type for AnchorResult<()>!
            quote! {
                msg!("Hello!");
                let taxable_errors = [#(#errors)*];
                let taxable_anchor_errors: Vec<anchor_lang::error::Error> = taxable_errors.iter().map(|err| (*err).into()).collect();
                let taxable_error_codes: Vec<u32> = taxable_anchor_errors
                    .iter()
                    .map(|err| {
                        if let anchor_lang::error::Error::AnchorError(anchor_err) = err {
                            return anchor_err.error_code_number;
                        } else {
                            return 0;
                        }
                    })
                    .collect();

                match result {
                    Ok(()) => {
                        msg!("definitely should not match Ok")
                    },
                    Err(err) => {
                        match err {
                            Error::AnchorError(anchor_err) => {
                                msg!("caught anchor error");
                                if taxable_error_codes.iter().any(|err_code| *err_code == anchor_err.error_code_number) {
                                    let base_fee: u64 = #base_fee;
                                    let lamports = base_fee.min(self.#payer.lamports());
                                    solana_program::program::invoke(
                                        &solana_program::system_instruction::transfer(
                                            &self.#payer.key(),
                                            &self.#pay_to.key(),
                                            lamports
                                        ),
                                        &[self.#payer.to_account_info(), self.#pay_to.to_account_info(), self.system_program.to_account_info()]
                                    )?;
                                };
                            }
                            _ => {
                                msg!("failed to match anchor error")
                            }
                        }
                    }
                }
            }
        }
    };

    quote! {
        #[automatically_derived]
        impl<#combined_generics> anchor_lang::Accounts<#trait_generics> for #name<#struct_generics> #where_clause {
            #[inline(never)]
            fn try_accounts(
                program_id: &anchor_lang::solana_program::pubkey::Pubkey,
                accounts: &mut &[anchor_lang::solana_program::account_info::AccountInfo<'info>],
                ix_data: &[u8],
                __bumps: &mut std::collections::BTreeMap<String, u8>,
            ) -> anchor_lang::Result<Self> {
                // Deserialize instruction, if declared.
                #ix_de
                // Deserialize each account.
                #(#deser_fields)*
                // Execute accounts constraints.
                #constraints
                // Success. Return the validated accounts.
                Ok(#accounts_instance)
            }

            #[inline(never)]
            fn handle_error(
                &self,
                result: anchor_lang::Result<()>,
            ) -> anchor_lang::Result<()> {
                #error_handling
                Ok(())
            }
        }
    }
}

pub fn generate_constraints(accs: &AccountsStruct) -> proc_macro2::TokenStream {
    let non_init_fields: Vec<&AccountField> =
        accs.fields.iter().filter(|af| !is_init(af)).collect();

    // Deserialization for each pda init field. This must be after
    // the inital extraction from the accounts slice and before access_checks.
    let init_fields: Vec<proc_macro2::TokenStream> = accs
        .fields
        .iter()
        .filter_map(|af| match af {
            AccountField::CompositeField(_s) => None,
            AccountField::Field(f) => match is_init(af) {
                false => None,
                true => Some(f),
            },
        })
        .map(constraints::generate)
        .collect();

    // Constraint checks for each account fields.
    let access_checks: Vec<proc_macro2::TokenStream> = non_init_fields
        .iter()
        .map(|af: &&AccountField| match af {
            AccountField::Field(f) => constraints::generate(f),
            AccountField::CompositeField(s) => constraints::generate_composite(s),
        })
        .collect();

    quote! {
        #(#init_fields)*
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

fn is_init(af: &AccountField) -> bool {
    match af {
        AccountField::CompositeField(_s) => false,
        AccountField::Field(f) => f.constraints.init.is_some(),
    }
}
