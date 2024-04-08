mod entities;
mod properties;

#[proc_macro_derive(QevyProperty, attributes(qevy_property))]
pub fn qevy_property_derive_macro(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    properties::qevy_property_derive_macro2(item.into())
        .unwrap()
        .into()
}

#[proc_macro_derive(QevyEntity, attributes(qevy_entity))]
pub fn qevy_entity_derive_macro(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    entities::qevy_entity_derive_macro2(item.into())
        .unwrap()
        .into()
}
