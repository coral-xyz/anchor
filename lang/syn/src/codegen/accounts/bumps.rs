use crate::*;

use super::*;

pub fn generate_bumps_name(anchor_ident: &Ident) -> Ident {
    Ident::new(&format!("{}Bumps", anchor_ident), Span::call_site())
}

pub fn generate(accs: &AccountsStruct) -> proc_macro2::TokenStream {
    let bumps_name = generate_bumps_name(&accs.ident);

    let bump_fields: Vec<proc_macro2::TokenStream> = accs
        .fields
        .iter()
        .filter_map(|af| {
            let constraints = match af {
                AccountField::Field(f) => constraints::linearize(&f.constraints),
                AccountField::CompositeField(s) => constraints::linearize(&s.constraints),
            };
            for c in constraints.iter() {
                let ident = af.ident();
                let bump_field = quote! {
                    pub #ident: u8
                };

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
        })
        .collect();

    quote! {
        #[derive(Default, Debug)]
        pub struct #bumps_name {
            #(#bump_fields),*
        }
    }
}
