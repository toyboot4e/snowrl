mod args;

use {darling::*, proc_macro2::TokenStream as TokenStream2, quote::*, syn::*};

/// Implements `TypeObject`
pub fn impl_tyobj(ast: syn::DeriveInput) -> TokenStream2 {
    let args = args::TypeArgs::from_derive_input(&ast).unwrap();

    match args.data {
        ast::Data::Struct(ref fields) => self::tyobj_struct(&args, fields),
        // ast::Data::Enum(ref fields) => self::inspect_unit_enum(&args, fields),
        _ => panic!(),
    }
}

fn tyobj_struct(args: &args::TypeArgs, _fields: &ast::Fields<args::FieldArgs>) -> TokenStream2 {
    let ty_ident = &args.ident;
    let (impl_generics, ty_generics, where_clause) = args.generics.split_for_impl();

    quote! {
        impl #impl_generics TypeObject for #ty_ident #ty_generics #where_clause {}
    }
}
