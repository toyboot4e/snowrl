//! Procedual macros

mod inspect;

use {
    proc_macro::TokenStream,
    syn::{parse_macro_input, DeriveInput},
};

#[proc_macro_derive(Inspect)]
pub fn inspect(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    inspect::impl_inspect(ast)
}
