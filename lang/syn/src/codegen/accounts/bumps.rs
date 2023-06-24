use crate::{
    codegen::accounts::{generics, ParsedGenerics},
    *,
};
use std::fmt::Display;

use super::constraints;

pub fn generate_bumps_name<T: Display>(anchor_ident: &T) -> Ident {
    Ident::new(&format!("{}Bumps", anchor_ident), Span::call_site())
}

pub fn generate(accs: &AccountsStruct) -> proc_macro2::TokenStream {
    let name = &accs.ident;
    let bumps_name = generate_bumps_name(name);
    let ParsedGenerics {
        combined_generics,
        trait_generics: _,
        struct_generics,
        where_clause,
    } = generics(accs);

    let bump_fields: Vec<proc_macro2::TokenStream> = accs
        .fields
        .iter()
        .filter_map(|af| {
            let ident = af.ident();

            match af {
                AccountField::Field(f) => {
                    let constraints = constraints::linearize(&f.constraints);
                    let bump_field = quote!(pub #ident: u8);

                    for c in constraints.iter() {
                        // Verify this in super::constraints
                        // The bump is only cached if
                        // - PDA is marked as init
                        // - PDA is not init, but marked with bump without a target

                        match c {
                            Constraint::Seeds(c) => {
                                if !c.is_init && c.bump.is_none() {
                                    return Some(bump_field);
                                }
                            }
                            Constraint::Init(c) => {
                                if c.seeds.is_some() {
                                    return Some(bump_field);
                                }
                            }
                            _ => (),
                        }
                    }
                    None
                }
                AccountField::CompositeField(s) => {
                    let comp_bumps_struct = generate_bumps_name(&s.symbol);
                    let bumps = quote!(pub #ident: #comp_bumps_struct);

                    Some(bumps)
                }
            }
        })
        .collect();

    quote! {
        #[derive(Default, Debug)]
        pub struct #bumps_name {
            #(#bump_fields),*
        }

        impl<#combined_generics> anchor_lang::Bumps for #name<#struct_generics> #where_clause {
            type Bumps = #bumps_name;
        }
    }
}
