mod entities;
mod properties;

#[proc_macro_derive(QevyProperty, attributes(qevy_property))]
pub fn qevy_property_derive_macro(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    properties::qevy_property_derive_macro2(item.into())
        .unwrap()
        .into()
}
