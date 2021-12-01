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

    let fields: Vec<_> = accs
        .fields
        .iter()
        .map(|f: &AccountField| match f {
            AccountField::CompositeField(s) => {
                let name = &s.ident;
                let name_string = format!("{}", name);
                quote! {
                    fields
                        .extend(
                            {
                                let mut temp = vec![];
                                anchor_lang::__private::fields::Fields::fields(&(self.#name), &mut temp);
                                temp
                                    .into_iter()
                                    .map(|mut field| {field.path.push(#name_string); field})
                            }
                        );
                }
            }
            AccountField::Field(f) => {
                let name = &f.ident;
                let name_string = name.to_token_stream().to_string();
                let dup_target = {
                    match &f.constraints.dup {
                        None => quote! { None },
                        Some(constraint_dup) => {
                            let target_name = &constraint_dup.target;
                            let target_name_string = target_name.to_token_stream().to_string();
                            quote! { Some(#target_name_string) }
                        }
                    }
                };
                let is_mutable = if &f.account_ty().to_token_stream().to_string() == "AccountInfo" {
                    quote! {self.#name.is_writable}
                } else {
                    quote! {
                        anchor_lang::IsMutable::is_mutable(&(self.#name))
                    }
                };
                quote! {
                    fields.push(anchor_lang::__private::fields::Field {
                        name: #name_string,
                        is_mutable: #is_mutable,
                        dup_target: #dup_target,
                        key: anchor_lang::Key::key(&(self.#name)),
                        path: vec![]
                    });
                }
            }
        })
        .collect();

    quote! {
        #[automatically_derived]
        impl<#combined_generics> anchor_lang::__private::fields::Fields for #name <#struct_generics> #where_clause{
            fn fields(&self, fields: &mut Vec<anchor_lang::__private::fields::Field>) {
                #(#fields)*
            }
        }
    }
}
