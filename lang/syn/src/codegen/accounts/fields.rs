use crate::codegen::accounts::{generics, ParsedGenerics};
use crate::{AccountField, AccountsStruct};
use quote::{quote, ToTokens};

// Generates the `Fields` trait implementation.
pub fn generate(accs: &AccountsStruct) -> proc_macro2::TokenStream {
    let name = &accs.ident;
    let ParsedGenerics {
        combined_generics: _,
        trait_generics: _,
        struct_generics,
        where_clause,
    } = generics(accs);

    let fields:Vec<_> = accs
        .fields
        .iter()
        .map(|f: &AccountField| match f {
            AccountField::CompositeField(s) => {
                let name = &s.ident;
                quote! { fields.extend(anchor_lang::__private::fields::Fields::fields(&(self.#name))); }
            }
            AccountField::Field(f) => {
                let name = &f.ident;
                let name_string = name.to_token_stream().to_string();
                let dup = {
                    match &f.constraints.dup {
                        None => quote! { None },
                        Some(constraint_dup) => {
                            let target_name = &constraint_dup.target;
                            let target_name_string = target_name.to_token_stream().to_string();
                            quote! { Some((#target_name_string, anchor_lang::Key::key(&(self.#target_name)))) }
                        }
                    }
                };
                let is_mutable = f.constraints.is_mutable();
                quote! {
                    fields.push(anchor_lang::__private::fields::Field {
                        name: #name_string,
                        address: anchor_lang::Key::key(&(self.#name)),
                        dup_target: #dup,
                        is_mutable: #is_mutable
                    });
                }
            }
        })
        .collect();

    quote! {
        #[automatically_derived]
        impl<#struct_generics> anchor_lang::__private::fields::Fields for #name <#struct_generics> #where_clause{
            fn fields(&self) -> Vec<anchor_lang::__private::fields::Field> {
                let mut fields = vec![];

                #(#fields)*

                fields
            }
        }
    }
}
