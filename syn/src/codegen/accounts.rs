use crate::{
    AccountField, AccountsStruct, Constraint, ConstraintBelongsTo, ConstraintLiteral,
    ConstraintOwner, ConstraintRentExempt, ConstraintSigner, Field, Ty,
};
use quote::quote;

pub fn generate(accs: AccountsStruct) -> proc_macro2::TokenStream {
    // Deserialization for each field.
    let deser_fields: Vec<proc_macro2::TokenStream> = accs
        .fields
        .iter()
        .map(|af: &AccountField| match af {
            AccountField::AccountsStruct(s) => {
                let name = &s.ident;
                quote! {
                    let #name = Accounts::try_accounts(program_id, accounts)?;
                }
            }
            AccountField::Field(f) => {
                let name = f.typed_ident();
                match f.is_init {
                    false => quote! {
                        let #name = Accounts::try_accounts(program_id, accounts)?;
                    },
                    true => quote! {
                        let #name = AccountsInit::try_accounts_init(program_id, accounts)?;
                    },
                }
            }
        })
        .collect();

    // Constraint checks for each account fields.
    let access_checks: Vec<proc_macro2::TokenStream> = accs
        .fields
        .iter()
        // TODO: allow constraints on composite fields.
        .filter_map(|af: &AccountField| match af {
            AccountField::AccountsStruct(_) => None,
            AccountField::Field(f) => Some(f),
        })
        .map(|f: &Field| {
            let checks: Vec<proc_macro2::TokenStream> = f
                .constraints
                .iter()
                .map(|c| generate_constraint(&f, c))
                .collect();
            quote! {
                #(#checks)*
            }
        })
        .collect();

    // Each field in the final deserialized accounts struct.
    let return_tys: Vec<proc_macro2::TokenStream> = accs
        .fields
        .iter()
        .map(|f: &AccountField| {
            let name = match f {
                AccountField::AccountsStruct(s) => &s.ident,
                AccountField::Field(f) => &f.ident,
            };
            quote! {
                #name
            }
        })
        .collect();

    // Exit program code-blocks for each account.
    let on_save: Vec<proc_macro2::TokenStream> = accs
        .fields
        .iter()
        .map(|af: &AccountField| {
            match af {
                AccountField::AccountsStruct(s) => {
                    let name = &s.ident;
                    quote! {
                        self.#name.exit(program_id)?;
                    }
                }
                AccountField::Field(f) => {
                    let ident = &f.ident;
                    let info = match f.ty {
                        // Only ProgramAccounts are automatically saved (when
                        // marked `#[account(mut)]`).
                        Ty::ProgramAccount(_) => quote! { #ident.to_account_info() },
                        _ => return quote! {},
                    };
                    match f.is_mut {
                        false => quote! {},
                        true => quote! {
                            // Only persist the change if the account is owned by the
                            // current program.
                            if program_id == self.#info.owner  {
                                let info = self.#info;
                                let mut data = info.try_borrow_mut_data()?;
                                let dst: &mut [u8] = &mut data;
                                let mut cursor = std::io::Cursor::new(dst);
                                self.#ident.try_serialize(&mut cursor)?;
                            }
                        },
                    }
                }
            }
        })
        .collect();

    // Implementation for `ToAccountInfos` trait.
    let to_acc_infos: Vec<proc_macro2::TokenStream> = accs
        .fields
        .iter()
        .map(|f: &AccountField| {
            let name = match f {
                AccountField::AccountsStruct(s) => &s.ident,
                AccountField::Field(f) => &f.ident,
            };
            quote! {
                account_infos.extend(self.#name.to_account_infos());
            }
        })
        .collect();

    // Implementation for `ToAccountMetas` trait.
    let to_acc_metas: Vec<proc_macro2::TokenStream> = accs
        .fields
        .iter()
        .map(|f: &AccountField| {
            let name = match f {
                AccountField::AccountsStruct(s) => &s.ident,
                AccountField::Field(f) => &f.ident,
            };
            quote! {
                account_metas.extend(self.#name.to_account_metas());
            }
        })
        .collect();

    let name = &accs.ident;
    let (combined_generics, trait_generics, strct_generics) = match accs.generics.lt_token {
        None => (quote! {<'info>}, quote! {<'info>}, quote! {}),
        Some(_) => {
            let g = &accs.generics;
            (quote! {#g}, quote! {#g}, quote! {#g})
        }
    };

    quote! {
        impl#combined_generics anchor_lang::Accounts#trait_generics for #name#strct_generics {
            fn try_accounts(program_id: &solana_program::pubkey::Pubkey, accounts: &mut &[solana_program::account_info::AccountInfo<'info>]) -> Result<Self, solana_program::program_error::ProgramError> {
                // Deserialize each account.
                #(#deser_fields)*

                // Perform constraint checks on each account.
                #(#access_checks)*

                // Success. Return the validated accounts.
                Ok(#name {
                    #(#return_tys),*
                })
            }
        }

        impl#combined_generics anchor_lang::ToAccountInfos#trait_generics for #name#strct_generics {
            fn to_account_infos(&self) -> Vec<solana_program::account_info::AccountInfo<'info>> {
                let mut account_infos = vec![];

                #(#to_acc_infos)*

                account_infos
            }
        }

        impl#combined_generics anchor_lang::ToAccountMetas for #name#strct_generics {
            fn to_account_metas(&self) -> Vec<solana_program::instruction::AccountMeta> {
                let mut account_metas = vec![];

                #(#to_acc_metas)*


                account_metas
            }
        }

        impl#strct_generics #name#strct_generics {
            pub fn exit(&self, program_id: &solana_program::pubkey::Pubkey) -> solana_program::entrypoint::ProgramResult {
                #(#on_save)*
                Ok(())
            }
        }
    }
}

pub fn generate_constraint(f: &Field, c: &Constraint) -> proc_macro2::TokenStream {
    match c {
        Constraint::BelongsTo(c) => generate_constraint_belongs_to(f, c),
        Constraint::Signer(c) => generate_constraint_signer(f, c),
        Constraint::Literal(c) => generate_constraint_literal(f, c),
        Constraint::Owner(c) => generate_constraint_owner(f, c),
        Constraint::RentExempt(c) => generate_constraint_rent_exempt(f, c),
    }
}

pub fn generate_constraint_belongs_to(
    f: &Field,
    c: &ConstraintBelongsTo,
) -> proc_macro2::TokenStream {
    // todo: assert the field type.

    let target = c.join_target.clone();
    let ident = &f.ident;
    // todo: would be nice if target could be an account info object.
    quote! {
        if &#ident.#target != #target.to_account_info().key {
            return Err(ProgramError::Custom(1)); // todo: error codes
        }
    }
}

pub fn generate_constraint_signer(f: &Field, _c: &ConstraintSigner) -> proc_macro2::TokenStream {
    let ident = &f.ident;
    let info = match f.ty {
        Ty::AccountInfo => quote! { #ident },
        Ty::ProgramAccount(_) => quote! { #ident.to_account_info() },
        _ => panic!("Invalid syntax: signer cannot be specified."),
    };
    quote! {
        if !#info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
    }
}

pub fn generate_constraint_literal(_f: &Field, c: &ConstraintLiteral) -> proc_macro2::TokenStream {
    let tokens = &c.tokens;
    quote! {
        if !(#tokens) {
            return Err(ProgramError::Custom(1)); // todo: error codes
        }
    }
}

pub fn generate_constraint_owner(f: &Field, c: &ConstraintOwner) -> proc_macro2::TokenStream {
    let ident = &f.ident;
    let info = match f.ty {
        Ty::AccountInfo => quote! { #ident },
        Ty::ProgramAccount(_) => quote! { #ident.to_account_info() },
        _ => panic!("Invalid syntax: owner cannot be specified."),
    };
    match c {
        ConstraintOwner::Skip => quote! {},
        ConstraintOwner::Program => quote! {
            if #info.owner != program_id {
                return Err(ProgramError::Custom(1)); // todo: error codes
            }
        },
    }
}

pub fn generate_constraint_rent_exempt(
    f: &Field,
    c: &ConstraintRentExempt,
) -> proc_macro2::TokenStream {
    let ident = &f.ident;
    let info = match f.ty {
        Ty::AccountInfo => quote! { #ident },
        Ty::ProgramAccount(_) => quote! { #ident.to_account_info() },
        _ => panic!("Invalid syntax: rent exemption cannot be specified."),
    };
    match c {
        ConstraintRentExempt::Skip => quote! {},
        ConstraintRentExempt::Enforce => quote! {
            if !rent.is_exempt(#info.lamports(), #info.try_data_len()?) {
                return Err(ProgramError::Custom(2)); // todo: error codes
            }
        },
    }
}
