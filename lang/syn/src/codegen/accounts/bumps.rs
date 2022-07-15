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
                if let Constraint::Seeds(..) = c {
                    let ident = af.ident();
                    return Some(quote! {
                        pub #ident: Option<u8>
                    });
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
