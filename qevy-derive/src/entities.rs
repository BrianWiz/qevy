use qevy_types::QevyEntityType;
use syn::{Attribute, DeriveInput, Meta, MetaNameValue};

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(qevy_entity))]
struct QevyEntityStructAttributes {
    #[deluxe(default = "Point".to_string())]
    entity_type: String,
    #[deluxe(default = "".to_string())]
    desc: String, // -> i'd like to replace this with doc comments!
}

pub(crate) fn qevy_entity_derive_macro2(
    item: proc_macro2::TokenStream,
) -> deluxe::Result<proc_macro2::TokenStream> {
    // Parse
    let mut ast: DeriveInput = syn::parse2(item)?;

    // Extract struct attributes
    let QevyEntityStructAttributes { entity_type, desc } = deluxe::extract_attributes(&mut ast)?;
    // only exists to check if the entity type is valid
    let _ = QevyEntityType::from_short_string(entity_type.as_str())
        .expect(format!("Invalid entity type: {}", entity_type).as_str());

    // Extract field attributes
    // TODO: do we need field attributes for entities? if so, what?
    // Description!! doc comments? yes

    // define impl variables
    let ident = &ast.ident;
    let (impl_generics, type_generics, where_clause) = ast.generics.split_for_impl();

    // Generate code
    let generated_code = quote::quote!(
        impl #impl_generics QevyEntity for #ident #type_generics #where_clause {
            fn get_description(&self) -> &str {
                #desc
            }

            fn get_entity_type(&self) -> QevyEntityType {
                QevyEntityType::from_short_string(#entity_type).unwrap()
            }
        }
    );

    Ok(generated_code)
}

fn get_comments(attrs: &[Attribute]) -> Option<String> {
    let mut docs = Vec::new();
    for attr in attrs {
        if let Meta::NameValue(MetaNameValue { path, value, .. }) = &attr.meta {
            if path.is_ident("doc") {
                return Some(syn::parse_quote!(#value))
            }
        }
    }
    None
}