use {proc_macro::TokenStream, proc_macro2::TokenStream as TokenStream2, quote::*, syn::*};

/// Implements `Inspect`
pub fn impl_inspect(ast: syn::DeriveInput) -> TokenStream {
    match ast.data {
        Data::Struct(ref data) => self::inspect_struct(data, &ast),
        Data::Enum(ref data) => self::inspect_enum(data, &ast),
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

    let ty_name = &ast.ident;

    self::generate_inspect_impl(
        ast,
        quote! {
            if !imgui::CollapsingHeader::new(&imgui::im_str!("{}", label))
                .default_open(true)
                .build(ui)
            {
                return;
            }

            ui.indent();
            #(#field_inspectors)*
            ui.unindent();
        },
    )
}

/// Call `inspect` for every field
fn collect_field_inspectors(fields: &Fields) -> impl Iterator<Item = TokenStream2> + '_ {
    let fields = match fields {
        Fields::Named(ref fields) => fields,
        _ => unimplemented!("`#[derive(Inspect)]` is only for struct with named fields"),
    };

    // TODO: skip attribute for fields
    fields.named.iter().map(|field| {
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
}

fn inspect_enum(data: &DataEnum, ast: &syn::DeriveInput) -> TokenStream {
    for v in &data.variants {
        assert!(
            v.fields.is_empty(),
            "Only plain enum variants are supported by `#[derive(Inspect)]`"
        );
    }

    let ty_name = &ast.ident;
    let variant_strings = data.variants.iter().map(|v| format!("{}", v.ident));

    let variant_idents = data
        .variants
        .iter()
        .map(|v| format_ident!("{}", v.ident))
        .collect::<Vec<_>>();

    self::generate_inspect_impl(
        ast,
        quote! {{
            fn item_ix(item: &#ty_name) -> Option<usize> {
                let items = [#(#ty_name::#variant_idents,)*];
                items
                    .iter()
                    .enumerate()
                    .find_map(|(i, item)| if item == item { Some(i) } else { None })
            }

            fn item_name(item: &#ty_name) -> Option<&'static imgui::ImStr> {
                let imgui_names: &[&'static imgui::ImStr] = &[
                    #(imgui::im_str!(#variant_strings),)*
                ];
                item_ix(item).map(|ix| &*imgui_names[ix])
            }

            let items = [#(#ty_name::#variant_idents,)*];
            let (mut ix, _v) = items
                .iter()
                .enumerate()
                .find(|(i, v)| *v == self)
                .unwrap();

            // FIXME: don't creaet labels dynamically
            if imgui::ComboBox::new(&imgui::im_str!("{}", label))
                .build_simple(
                    ui,
                    &mut ix,
                    &items,
                    &|v| std::borrow::Cow::Borrowed(item_name(v).unwrap()),
                ) {
                    *self = items[ix].clone();
            }
        }},
    )
}

