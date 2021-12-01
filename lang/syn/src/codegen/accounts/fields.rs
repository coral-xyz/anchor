use crate::codegen::accounts::{generics, ParsedGenerics};
use crate::{AccountField, AccountsStruct};
use quote::{quote, ToTokens};

// Generates the `Fields` trait implementation.
pub fn generate(accs: &AccountsStruct) -> proc_macro2::TokenStream {
    let name = &accs.ident;
    let ParsedGenerics {
        combined_generics,
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
                quote! {
                    fields.push(anchor_lang::__private::fields::Field {
                        name: #name_string,
                        address: anchor_lang::Key::key(&(self.#name)),
                        is_mutable: std::convert::AsRef::as_ref(&(self.#name)).is_writable
                    });
                }
            }
        })
        .collect();

    quote! {
        #[automatically_derived]
        impl<#combined_generics> anchor_lang::__private::fields::Fields for #name <#struct_generics> #where_clause{
            fn fields(&self) -> Vec<anchor_lang::__private::fields::Field> {
                let mut fields = vec![];

                #(#fields)*

                fields
            }
        }
    }
}
