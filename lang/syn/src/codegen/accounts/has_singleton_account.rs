use crate::{AccountField, AccountsStruct, Ty};
use quote::quote;

/// Generates the [anchor_lang::HasSingletonAccount] trait implementations.
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
                    let account_ty = f.account_ty();
                    match &f.ty {
                        Ty::Program(_) | Ty::Sysvar(_) => {
                            quote! {
                                #[automatically_derived]
                                impl<'info> anchor_lang::HasSingletonAccount<'info, #account_ty> for #name<'info> {
                                    fn instance(&self) -> anchor_lang::solana_program::account_info::AccountInfo<'info> {
                                        self.#field_name.to_account_info()
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
