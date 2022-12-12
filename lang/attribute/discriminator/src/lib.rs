extern crate proc_macro;

/// A marker attribute used to override the discriminator value that should be used.
#[proc_macro_attribute]
pub fn discriminator(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    input
}
