mod args;

use {darling::*, proc_macro2::TokenStream as TokenStream2, quote::*, syn::*};

pub fn impl_via_tyobj(ast: syn::DeriveInput) -> TokenStream2 {
    let args = args::TypeArgs::from_derive_input(&ast).unwrap();

    match args.data {
        ast::Data::Struct(ref fields) => self::via_tyobj_struct(&args, fields),
        // ast::Data::Enum(ref fields) => self::inspect_unit_enum(&args, fields),
        _ => panic!(),
    }
}

fn via_tyobj_struct(args: &args::TypeArgs, _fields: &ast::Fields<args::FieldArgs>) -> TokenStream2 {
    let ty_ident = &args.ident;
    let (impl_generics, ty_generics, where_clause) = args.generics.split_for_impl();

    let tyobj = &args.tyobj;
    let from_tyobj = &args.from_tyobj;

    let root = quote!(snow2d::utils::tyobj);

    let repr_fns = args.repr_field.as_ref().map(|repr| {
        quote! {
            fn _repr_mut(&mut self) -> Option<&mut #root::SerdeRepr<Self::TypeObject>> {
                Some(&mut self.#repr)
            }

            fn into_tyobj_repr(self) -> Option<#root::SerdeRepr<Self::TypeObject>>
            where
                Self: Sized,
            {
                Some(self.#repr)
            }
        }
    });

    let into_impl = args.repr_field.as_ref().map(|repr| {
        quote! {
            // Target -> SerdeRepr<TypeObject>
            impl #impl_generics Into<#root::SerdeRepr<#tyobj>> for #ty_ident #ty_generics
                #where_clause
            {
                fn into(self: #ty_ident #ty_generics) -> #root::SerdeRepr<#tyobj> {
                    <#ty_ident as #root::SerdeViaTyObj>::into_tyobj_repr(self)
                        .unwrap_or_else(|| unreachable!())
                }
            }
        }
    });

    quote! {
        impl #impl_generics SerdeViaTyObj for #ty_ident #ty_generics
            #where_clause
        {
            type TypeObject = #tyobj;

            fn _from_tyobj(obj: &Self::TypeObject) -> Self {
                SerdeViaTyObj::_from_tyobj(obj)
            }

            #repr_fns
        }

        // SerdeRepr<TypeObject> -> Target
        impl #impl_generics From<#root::SerdeRepr<#tyobj>> for #ty_ident #ty_generics
            #where_clause
        {
            fn from(repr: #root::SerdeRepr<#tyobj>) -> #ty_ident #ty_generics {
                SerdeViaTyObj::from_tyobj_repr(repr)
            }
        }

        #into_impl
    }
}
