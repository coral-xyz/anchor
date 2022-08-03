use crate::codegen::accounts::{generics, ParsedGenerics};
use crate::{AccountField, AccountsStruct};
use quote::quote;

// Generates the `TryAccountInfos` trait implementation.
pub fn generate(accs: &AccountsStruct) -> proc_macro2::TokenStream {
    let name = &accs.ident;
    let ParsedGenerics {
        combined_generics,
        trait_generics,
        struct_generics,
        where_clause,
    } = generics(accs);

    let try_acc_infos: Vec<proc_macro2::TokenStream> = accs
        .fields
        .iter()
        .map(|f: &AccountField| {
            let name = &f.ident();
            quote! { account_infos.extend(self.#name.try_account_infos(program)); }
        })
        .collect();
    quote! {
        #[automatically_derived]
        impl<#combined_generics> anchor_lang::TryAccountInfos<#trait_generics> for #name <#struct_generics> #where_clause{
            fn try_account_infos(&self, program: &anchor_lang::solana_program::account_info::AccountInfo<'info>) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
                let mut account_infos = vec![];

                #(#try_acc_infos)*

                account_infos
            }
        }
    }
}
