extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};


/// An attribute for benching instruction.
/// 
/// A marker attribute used to increase the stack frame of the instruction,
/// to being able to output the warn message of the compiler containing how much size
/// has been allocated to the stack.
#[proc_macro_attribute]
pub fn bench_ix(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let func_ident= parse_macro_input!(input as ItemFn);
    let sig_item = func_ident.sig;

    let func_name = sig_item.ident;
    let arguments =  sig_item.inputs;
    let return_type = sig_item.output;
    let block = func_ident.block;

    let output = quote! {
        pub fn #func_name ( #arguments ) #return_type {
            // i64 type size 8b
            // 8b * 1023 = 8184b
            let big_var: [i64; 1023] = [10; 1023];
            msg!("{}", big_var.len());
            #block
        }
    };

    output.into()
}