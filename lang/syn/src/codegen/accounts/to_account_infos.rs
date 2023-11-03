use crate::codegen::accounts::{generics, ParsedGenerics};
use crate::{AccountField, AccountsStruct};
use quote::quote;

// Generates the `ToAccountInfos` trait implementation.
pub fn generate(accs: &AccountsStruct) -> proc_macro2::TokenStream {
    let name = &accs.ident;
    let ParsedGenerics {
        combined_generics,
        trait_generics,
        struct_generics,
        where_clause,
    } = generics(accs);

    let to_acc_infos: Vec<proc_macro2::TokenStream> = accs
        .fields
        .iter()
        .map(|f: &AccountField| {
            let name = &f.ident();
            quote! { account_infos.push(self.#name.to_account_info()); }
        })
        .collect();

    let length = to_acc_infos.len();

    quote! {
        #[automatically_derived]
        impl<#combined_generics> anchor_lang::ToAccountInfos<#trait_generics> for #name <#struct_generics> #where_clause{
            fn to_account_infos(&self) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
                use anchor_lang::ToAccountInfo;

                let mut account_infos = Vec::with_capacity(#length);

                #(#to_acc_infos)*

                account_infos
            }
        }
    }
}
