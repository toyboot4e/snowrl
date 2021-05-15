mod args;
mod utils;

use {darling::*, proc_macro2::TokenStream as TokenStream2, quote::*, syn::*};

/// Implements `Inspect`
pub fn impl_inspect(ast: syn::DeriveInput) -> TokenStream2 {
    let args = args::TypeArgs::from_derive_input(&ast).unwrap();

    match args.data {
        ast::Data::Struct(ref fields) => self::inspect_struct(&args, fields),
        ast::Data::Enum(ref fields) => self::inspect_enum(&args, fields),
    }
}

fn create_impl_generics(args: &args::TypeArgs) -> Generics {
    let mut generics = args.generics.clone();

    generics.make_where_clause().predicates.extend(
        args.all_fields()
            .iter()
            .filter(|f| !f.skip)
            .map(|f| &f.ty)
            .map::<WherePredicate, _>(|ty| parse_quote! { #ty: Inspect }),
    );

    generics
}

/// Fill the `inspect` function body to derive `Inspect`
fn generate_inspect_impl(args: &args::TypeArgs, inspect_body: TokenStream2) -> TokenStream2 {
    let generics = self::create_impl_generics(args);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let ty_ident = &args.ident;

    quote! {
        impl #impl_generics Inspect for #ty_ident #ty_generics #where_clause
        {
            fn inspect(&mut self, ui: &imgui::Ui, label: &str) {
                #inspect_body
            }
        }
    }
}

fn inspect_struct(args: &args::TypeArgs, fields: &ast::Fields<args::FieldArgs>) -> TokenStream2 {
    let inspect = if let Some(as_) = args.as_.as_ref() {
        let as_: Type = parse_str(as_).unwrap();

        quote! {
            let mut x: #as_ = (*self).into();
            x.inspect(ui, label);
            *self = x.into();
        }
    } else {
        let is_newtype =
            fields.style == ast::Style::Tuple && fields.iter().filter(|x| !x.skip).count() == 1;
        if is_newtype {
            // delegate the inspection to the only field
            quote! {
                self.0.inspect(ui, label);
            }
        } else if args.in_place {
            // inspect each field
            let field_inspectors = utils::struct_field_inspectors(&fields);

            quote! {
                #(#field_inspectors)*
            }
        } else {
            // insert tree and inspect each field
            let field_inspectors = utils::struct_field_inspectors(&fields);

            let open = args.open;
            quote! {
                imgui::TreeNode::new(&imgui::im_str!("{}", label))
                    .flags(
                        imgui::TreeNodeFlags::OPEN_ON_ARROW |
                        imgui::TreeNodeFlags::OPEN_ON_DOUBLE_CLICK
                    )
                    .default_open(#open)
                    .build(ui, ||
                           {
                               #(#field_inspectors)*
                           })
            }
        }
    };

    self::generate_inspect_impl(args, inspect)
}

fn inspect_enum(args: &args::TypeArgs, variants: &[args::VariantArgs]) -> TokenStream2 {
    if variants.iter().all(|v| v.fields.is_empty()) {
        self::inspect_unit_enum(args, variants)
    } else {
        self::inspect_enum_variant(args, variants)
    }
}

/// Inspect the variant's fields
fn inspect_enum_variant(args: &args::TypeArgs, variants: &[args::VariantArgs]) -> TokenStream2 {
    let ty_ident = &args.ident;

    let matchers = variants.iter().map(|v| {
        let v_ident = &v.ident;

        match v.fields.style {
            ast::Style::Struct => {
                let f_idents = v
                    .fields
                    .iter()
                    .map(|f| {
                        let ident = &f.ident;
                        quote!(#ident)
                    })
                    .collect::<Vec<_>>();

                let labels = v
                    .fields
                    .iter()
                    .map(|f| format!("{}", f.ident.as_ref().unwrap()));

                quote! {
                    #ty_ident::#v_ident { #(#f_idents),* } => {
                        #(#f_idents.inspect(ui, #labels);)*
                    }
                }
            }
            ast::Style::Tuple => {
                let f_idents = (0..v.fields.len())
                    .map(|i| format_ident!("f{}", i))
                    .collect::<Vec<_>>();
                let labels = (0..v.fields.len()).map(|i| format!("{}", i));

                quote! {
                    #ty_ident::#v_ident(#(#f_idents),*) => {
                        #(#f_idents.inspect(ui, #labels);)*
                    }
                }
            }
            ast::Style::Unit => quote! {
                #ty_ident::#v_ident
            },
        }
    });

    self::generate_inspect_impl(
        args,
        quote! {{
            match self {
                #(#matchers,)*
            }
        }},
    )
}

/// Show menu to choose one of the variants
fn inspect_unit_enum(args: &args::TypeArgs, variants: &[args::VariantArgs]) -> TokenStream2 {
    let ty_ident = &args.ident;

    // create `[TypeName::A, TypeName::B]`
    let variant_idents = variants
        .iter()
        .map(|v| format_ident!("{}", v.ident))
        .collect::<Vec<_>>();

    // create `[im_str!("A"), im_str!("B")]
    let variant_names = variants.iter().map(|v| format!("{}", v.ident));

    self::generate_inspect_impl(
        args,
        quote! {{
            const VARIANTS: &[#ty_ident] = &[#(#ty_ident::#variant_idents,)*];

            fn item_ix(variant: &#ty_ident) -> Option<usize> {
                VARIANTS
                    .iter()
                    .enumerate()
                    .find_map(|(i, v)| if v == variant { Some(i) } else { None })
            }

            let imgui_names: &[&'static imgui::ImStr] = &[
                #(imgui::im_str!(#variant_names),)*
            ];

            let mut ix = item_ix(self).unwrap();
            let index = ix.clone();

            if imgui::ComboBox::new(&imgui::im_str!("{}", label))
                .build_simple(
                    ui,
                    &mut ix,
                    VARIANTS,
                    &|v| {
                        let i = item_ix(v).unwrap();
                        std::borrow::Cow::Borrowed(imgui_names[i])
                    },
                ) {
                    *self = VARIANTS[ix].clone();
            }
        }},
    )
}
