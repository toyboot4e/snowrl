/*!
Procedual macros

`Inspect` macro does nothing if `dummy` feature is specified.
*/

#[cfg(not(feature = "dummy"))]
mod inspect;

mod tyobj;
mod via_tyobj;

use {
    proc_macro::TokenStream,
    syn::{parse_macro_input, DeriveInput},
};

/// Implements `Inspect` trait
#[cfg(not(feature = "dummy"))]
#[proc_macro_derive(Inspect, attributes(inspect))]
pub fn inspect(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    TokenStream::from(inspect::impl_inspect(ast))
}

/// Implements `Inspect` trait
#[cfg(feature = "dummy")]
#[proc_macro_derive(Inspect, attributes(inspect))]
pub fn inspect(input: TokenStream) -> TokenStream {
    TokenStream::new()
}

/// Implements `TypeObject` trait
#[proc_macro_derive(TypeObject)]
pub fn tyobj(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    TokenStream::from(tyobj::impl_tyobj(ast))
}

/// Implements `SerdeViaTyObj` trait
#[proc_macro_derive(SerdeViaTyObj, attributes(via_tyobj))]
pub fn serde_via_tyobj(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    TokenStream::from(via_tyobj::impl_via_tyobj(ast))
}
