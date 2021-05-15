use darling::*;
use proc_macro2::TokenStream as TokenStream2;
use quote::*;
use syn::*;

use crate::inspect::args;

/// `self.field.inspect(ui, label);`
pub fn struct_field_inspectors<'a>(
    field_args: &'a ast::Fields<args::FieldArgs>,
) -> impl Iterator<Item = TokenStream2> + 'a {
    field_args
        .fields
        .iter()
        .filter(|field| !field.skip)
        .enumerate()
        .map(move |(field_index, field)| {
            let (field_ident, label) = match field_args.style {
                ast::Style::Struct => {
                    let field_ident = field.ident.as_ref().unwrap_or_else(|| unreachable!());
                    (quote!(#field_ident), format!("{}", field_ident))
                }
                ast::Style::Tuple => {
                    // `self.0`, not `self.0usize` for example
                    let field_ident = Index::from(field_index);
                    (quote!(#field_ident), format!("{}", field_index))
                }
                ast::Style::Unit => unreachable!(),
            };

            if let Some(as_) = field.as_.as_ref() {
                // inspect via converted type
                let as_: Type = parse_str(as_).unwrap();
                quote! {
                    {
                        let mut as_: #as_ = self.into();
                        as_.#field_ident.inspect(ui, #label);
                        *self = as_.into();
                    }
                }
            } else {
                // inspect the value as-is
                quote! {
                    self.#field_ident.inspect(ui, #label);
                }
            }
        })
}
