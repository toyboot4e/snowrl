use darling::*;
use syn::*;

#[derive(FromDeriveInput)]
#[darling(attributes(inspect), supports(struct_any, enum_any))]
pub struct TypeArgs {
    pub ident: Ident,
    pub generics: Generics,
    pub data: ast::Data<VariantArgs, FieldArgs>,
}

#[derive(FromField, Clone)]
#[darling(attributes(inspect))]
pub struct FieldArgs {
    pub ident: Option<Ident>,
    pub ty: Type,
}

#[derive(FromVariant)]
#[darling(attributes(inspect))]
pub struct VariantArgs {
    pub ident: Ident,
    pub fields: ast::Fields<FieldArgs>,
}

impl TypeArgs {
    /// Enumerates all the fields of a struct or enum variants
    #[allow(unused)]
    pub fn all_fields(&self) -> Vec<self::FieldArgs> {
        match &self.data {
            ast::Data::Struct(field_args) => field_args.fields.clone(),
            ast::Data::Enum(variants) => variants
                .iter()
                .flat_map(|variant| variant.fields.clone().into_iter())
                .collect::<Vec<_>>(),
        }
    }
}
