mod args;

use {darling::*, proc_macro2::TokenStream as TokenStream2, quote::*, syn::*};

/// Implements `Inspect`
pub fn impl_inspect(ast: syn::DeriveInput) -> TokenStream2 {
    let args = args::TypeArgs::from_derive_input(&ast).unwrap();

    match args.data {
        ast::Data::Struct(ref fields) => self::inspect_struct(&args, fields),
        ast::Data::Enum(ref fields) => self::inspec_unit_enum(&args, fields),
    }
}

/// Fill the `inspect` function body to derive `Inspect`
fn generate_inspect_impl(
    ty_name: &Ident,
    generics: &Generics,
    inspect_body: TokenStream2,
) -> TokenStream2 {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // TODO: add where clause

    quote! {
        impl #impl_generics snow2d::utils::Inspect for #ty_name #ty_generics #where_clause
        {
            fn inspect(&mut self, ui: &imgui::Ui, label: &str) {
                #inspect_body
            }
        }
    }
}

fn inspect_struct(args: &args::TypeArgs, fields: &ast::Fields<args::FieldArgs>) -> TokenStream2 {
    if let Some(as_) = args.as_.as_ref() {
        let as_: Type = parse_str(as_).unwrap();

        self::generate_inspect_impl(
            &args.ident,
            &args.generics,
            quote! {
                let mut x: #as_ = (*self).into();
                x.inspect(ui, label);
                *self = x.into();
            },
        )
    } else {
        let is_newtype = fields.style == ast::Style::Tuple && fields.len() == 1;
        if is_newtype {
            // delegate the inspection to the only field
            self::generate_inspect_impl(
                &args.ident,
                &args.generics,
                quote! {
                    self.0.inspect(ui, label);
                },
            )
        } else {
            // inspect each field
            let field_inspectors = self::collect_field_inspectors(&fields);

            self::generate_inspect_impl(
                &args.ident,
                &args.generics,
                quote! {
                    imgui::TreeNode::new(&imgui::im_str!("{}", label))
                        .flags(
                            imgui::TreeNodeFlags::OPEN_ON_ARROW |
                            imgui::TreeNodeFlags::OPEN_ON_DOUBLE_CLICK
                        )
                        .build(ui, ||
                    {
                        #(#field_inspectors)*
                    })
                },
            )
        }
    }
}

/// `self.field.inspect(ui, label);`
fn collect_field_inspectors<'a>(
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
                let as_: Type = parse_str(as_).unwrap();
                quote! {
                    {
                        let mut as_: #as_ = self.into();
                        as_.#field_ident.inspect(ui, #label);
                        *self = as_.into();
                    }
                }
            } else {
                quote! {
                    self.#field_ident.inspect(ui, #label);
                }
            }
        })
}

fn inspec_unit_enum(args: &args::TypeArgs, variants: &[args::VariantArgs]) -> TokenStream2 {
    for v in variants {
        assert!(
            v.fields.is_empty(),
            "Only plain enum variants are supported by `#[derive(Inspect)]`"
        );
    }

    let ty_name = &args.ident;

    // create `[TypeName::A, TypeName::B]
    let variant_idents = variants
        .iter()
        .map(|v| format_ident!("{}", v.ident))
        .collect::<Vec<_>>();

    // create `[im_str!("A"), im_str!("B")]
    let variant_names = variants.iter().map(|v| format!("{}", v.ident));

    self::generate_inspect_impl(
        &args.ident,
        &args.generics,
        quote! {{
            const VARIANTS: &[#ty_name] = &[#(#ty_name::#variant_idents,)*];

            fn item_ix(variant: &#ty_name) -> Option<usize> {
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
