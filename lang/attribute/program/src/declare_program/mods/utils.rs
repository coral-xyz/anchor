use anchor_idl::types::Idl;
use quote::{format_ident, quote};

use super::common::gen_discriminator;

pub fn gen_utils_mod(idl: &Idl) -> proc_macro2::TokenStream {
    let event = gen_event(idl);

    quote! {
        /// Program utilities.
        pub mod utils {
            #event
        }
    }
}

fn gen_event(idl: &Idl) -> proc_macro2::TokenStream {
    let variants = idl
        .events
        .iter()
        .map(|ev| format_ident!("{}", ev.name))
        .map(|name| quote! { #name(#name) });
    let match_arms = idl.events.iter().map(|ev| {
        let disc = gen_discriminator(&ev.discriminator);
        let name = format_ident!("{}", ev.name);
        let event = quote! { #name::try_from_slice(&value[8..]).map(Self::#name) };
        quote! { #disc => #event }
    });

    quote! {
        use super::{*, events::*};

        /// An enum that includes all events of the declared program as a tuple variant.
        ///
        /// See [`Self::try_from_bytes`] to create an instance from bytes.
        pub enum Event {
            #(#variants,)*
        }

        impl Event {
            /// Try to create an event based on the given bytes.
            ///
            /// This method returns an error if the discriminator of the given bytes don't match
            /// with any of the existing events, or if the deserialization fails.
            pub fn try_from_bytes(bytes: &[u8]) -> std::io::Result<Self> {
                Self::try_from(bytes)
            }
        }

        impl TryFrom<&[u8]> for Event {
            type Error = std::io::Error;

            fn try_from(value: &[u8]) -> std::io::Result<Self> {
                if value.len() < 8 {
                    return Err(std::io::ErrorKind::InvalidData.into());
                }

                match &value[..8] {
                    #(#match_arms,)*
                    _ => Err(std::io::ErrorKind::NotFound.into()),
                }
            }
        }
    }
}
