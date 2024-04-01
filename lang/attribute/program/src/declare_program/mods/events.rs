use anchor_syn::idl::types::Idl;
use quote::{format_ident, quote};

use super::common::{convert_idl_type_def_to_ts, gen_discriminator};

pub fn gen_events_mod(idl: &Idl) -> proc_macro2::TokenStream {
    let events = idl.events.iter().map(|ev| {
        let name = format_ident!("{}", ev.name);
        let discriminator = gen_discriminator(&ev.discriminator);

        let ty_def = idl
            .types
            .iter()
            .find(|ty| ty.name == ev.name)
            .map(|ty| convert_idl_type_def_to_ts(ty, &idl.types))
            .expect("Type must exist");

        quote! {
            #[derive(anchor_lang::__private::EventIndex)]
            #ty_def

            impl anchor_lang::Event for #name {
                fn data(&self) -> Vec<u8> {
                    let mut data = Vec::with_capacity(256);
                    data.extend_from_slice(&#discriminator);
                    self.serialize(&mut data).unwrap();
                    data
                }
            }

            impl anchor_lang::Discriminator for #name {
                const DISCRIMINATOR: [u8; 8] = #discriminator;
            }
        }
    });

    quote! {
        /// Program event type definitions.
        pub mod events {
            use super::{*, types::*};

            #(#events)*
        }
    }
}
