use crate::{AccountField, AccountsStruct, Ty};
use quote::quote;

// Generates the [HasSingleton] trait implementation.
pub fn generate(accs: &AccountsStruct) -> proc_macro2::TokenStream {
    let name = &accs.ident;

    let has_singletons: Vec<proc_macro2::TokenStream> = accs
        .fields
        .iter()
        .map(|f: &AccountField| {
            match f {
                AccountField::CompositeField(_) => quote! {},
                AccountField::Field(f) => {
                    let field_name = &f.ident;
                    match &f.ty {
                        Ty::Program(ty) => {
                            let path = &ty.account_type_path;
                            quote! {
                                #[automatically_derived]
                                impl anchor_lang::HasSingletonAccount<#path> for #name {
                                    fn get_account_info(&self) -> anchor_lang::solana_program::account_info::AccountInfo<'info> {
                                        self.#field_name
                                    }
                                }
                            }
                        },
                        _ => quote!{}
                    }
                }
            }
        })
        .collect();

    quote! {
        #(#has_singletons)*
    }
}
