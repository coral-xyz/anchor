use crate::codegen::accounts::{generics, ParsedGenerics};
use crate::{AccountField, AccountsStruct};
use quote::quote;

// Generates the `ToOptionalAccountInfos` trait implementation.
pub fn generate(accs: &AccountsStruct) -> proc_macro2::TokenStream {
    let name = &accs.ident;
    let ParsedGenerics {
        combined_generics,
        trait_generics,
        struct_generics,
        where_clause,
    } = generics(accs);

    let to_optional_acc_infos: Vec<proc_macro2::TokenStream> = accs
        .fields
        .iter()
        .map(|f: &AccountField| {
            let (name, optional) = match f {
                AccountField::CompositeField(s) => (&s.ident, false),
                AccountField::Field(f) => (&f.ident, f.optional),
            };
            if optional {
                quote! {
                    optional_account_infos.extend(self.#name.to_optional_account_infos(program));
                }
            } else {
                quote! {
                    optional_account_infos.extend(self.#name.to_account_infos());
                }
            }
        })
        .collect();
    quote! {
        #[automatically_derived]
        impl<#combined_generics> anchor_lang::ToOptionalAccountInfos<#trait_generics> for #name <#struct_generics> #where_clause{
            fn to_optional_account_infos(&self, program: &anchor_lang::solana_program::account_info::AccountInfo<'info>) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
                let mut optional_account_infos = vec![];

                #(#to_optional_acc_infos)*

                optional_account_infos
            }
        }
    }
}
