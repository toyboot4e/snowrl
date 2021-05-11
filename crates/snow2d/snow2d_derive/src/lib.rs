//! Procedual macros

mod inspect;

use {
    proc_macro::TokenStream,
    syn::{parse_macro_input, DeriveInput},
};

#[cfg(not(feature = "dummy"))]
#[proc_macro_derive(Inspect, attributes(inspect))]
pub fn inspect(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    TokenStream::from(inspect::impl_inspect(ast))
}

#[cfg(feature = "dummy")]
pub fn inspect(input: TokenStream) -> TokenStream {
    TokenStream::new()
}
