use crate::Program;
use quote::quote;

mod accounts;
pub mod common;
mod cpi;
mod declare_id;
mod dispatch;
mod entry;
mod handlers;
mod instruction;

pub fn generate(program: &Program) -> proc_macro2::TokenStream {
    let mod_name = &program.name;

    let _declare_id = declare_id::generate(program);
    let entry = entry::generate(program);
    let dispatch = dispatch::generate(program);
    let handlers = handlers::generate(program);
    let user_defined_program = &program.program_mod;
    let instruction = instruction::generate(program);
    let cpi = cpi::generate(program);
    let accounts = accounts::generate(program);

    quote! {
        // TODO: remove once we allow segmented paths in `Accounts` structs.
        use self::#mod_name::*;

        #entry
        #dispatch
        #handlers
        #user_defined_program
        #instruction
        #cpi
        #accounts
    }
}
