use crate::codegen::accounts::{generics, ParsedGenerics};
use crate::{AccountField, AccountsStruct};
use quote::quote;

// Generates the `ToAccountMetas` trait implementation.
pub fn generate(accs: &AccountsStruct) -> proc_macro2::TokenStream {
    let name = &accs.ident;
    let ParsedGenerics {
        combined_generics: _,
        trait_generics: _,
        struct_generics,
        where_clause: _,
    } = generics(accs);

    let to_acc_metas: Vec<proc_macro2::TokenStream> = accs
        .fields
        .iter()
        .map(|f: &AccountField| {
            let (name, is_signer) = match f {
                AccountField::CompositeField(s) => (&s.ident, quote! {None}),
                AccountField::Field(f) => {
                    let is_signer = match f.constraints.is_signer() {
                        false => quote! {None},
                        true => quote! {Some(true)},
                    };
                    (&f.ident, is_signer)
                }
            };
            quote! {
                account_metas.extend(self.#name.to_account_metas(#is_signer));
            }
        })
        .collect();

    let where_clause = &accs.generics.where_clause;

    quote! {
        impl<#struct_generics> anchor_lang::ToAccountMetas for #name <#struct_generics> #where_clause{
            fn to_account_metas(&self, is_signer: Option<bool>) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                let mut account_metas = vec![];

                #(#to_acc_metas)*

                account_metas
            }
        }
    }
}
