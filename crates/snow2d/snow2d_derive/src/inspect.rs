use {proc_macro::TokenStream, proc_macro2::TokenStream as TokenStream2, quote::*, syn::*};

/// Implements `Inspect`
pub fn impl_inspect(ast: syn::DeriveInput) -> TokenStream {
    match ast.data {
        Data::Struct(ref data) => self::inspect_struct(data, &ast),
        Data::Enum(ref data) => self::inspec_unit_enum(data, &ast),
        _ => panic!("`#[derive(VertexLayout)]` is for structs or enums"),
    }
}

/// Fill the `inspect` function body to derive `Inspect`
fn generate_inspect_impl(ast: &DeriveInput, inspect_body: TokenStream2) -> TokenStream {
    let ty_name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    TokenStream::from(quote! {
        impl #impl_generics snow2d::utils::Inspect for #ty_name #ty_generics #where_clause
        {
            fn inspect(&mut self, ui: &imgui::Ui, label: &str) {
                #inspect_body
            }
        }
    })
}

fn inspect_struct(data: &DataStruct, ast: &syn::DeriveInput) -> TokenStream {
    let field_inspectors = self::collect_field_inspectors(&data.fields);

    self::generate_inspect_impl(
        ast,
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

/// Call `inspect` for every field
fn collect_field_inspectors(fields: &Fields) -> Vec<TokenStream2> {
    // TODO: skip attribute for fields
    match fields {
        // `A { a: f32 }`
        Fields::Named(ref fields) => fields
            .named
            .iter()
            .map(|field| {
                // let field_ty = &field.ty;
                let field_name = field
                    .ident
                    .as_ref()
                    // TODO: print `name: Type in Type`
                    .unwrap_or_else(|| panic!("field name is required to derivie Inspect"));

                quote! {
                    self.#field_name.inspect(ui, stringify!(#field_name));
                }
            })
            .collect::<Vec<_>>(),
        // `A(f32);`
        Fields::Unnamed(ref fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, _field)| {
                // FIXME: is this correct name
                let i_name = format!("{}", i);
                // use `0` not `0usize` for example
                let i = Index::from(i);
                quote! {
                    self.#i.inspect(ui, #i_name);
                }
            })
            .collect::<Vec<_>>(),
        // `A,`
        Fields::Unit => {
            todo!("support unit structs");
        }
    }
}

fn inspec_unit_enum(data: &DataEnum, ast: &syn::DeriveInput) -> TokenStream {
    for v in &data.variants {
        assert!(
            v.fields.is_empty(),
            "Only plain enum variants are supported by `#[derive(Inspect)]`"
        );
    }

    let ty_name = &ast.ident;

    // create `[TypeName::A, TypeName::B]
    let variant_idents = data
        .variants
        .iter()
        .map(|v| format_ident!("{}", v.ident))
        .collect::<Vec<_>>();

    // create `[im_str!("A"), im_str!("B")]
    let variant_names = data.variants.iter().map(|v| format!("{}", v.ident));

    self::generate_inspect_impl(
        ast,
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
