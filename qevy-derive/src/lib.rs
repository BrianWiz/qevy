use std::collections::HashMap;

use syn::{DeriveInput, Ident};

fn impl_qevy_property_trait(ast: DeriveInput) -> proc_macro::TokenStream {
    // get struct identifier
    let ident: Ident = ast.ident;
    let ident_str = ident.to_string();

    // Get field identifiers
    let variant_idents = match ast.data {
        syn::Data::Struct(_) => panic!("QevyProperty cannot be derived for structs"),
        syn::Data::Enum(data) => data
            .variants
            .into_iter()
            .map(|variant| variant.ident)
            .collect::<Vec<Ident>>(),
        syn::Data::Union(_) => panic!("QevyProperty cannot be derived for unions"),
    };
    let variant_idents_str = variant_idents
        .iter()
        .map(|ident| ident.to_string())
        .collect::<Vec<String>>();

    // generate impl
    quote::quote!(
        impl QevyProperty for #ident {
            fn get_fgd_string(&self, field_name: &str) -> String {
                let mut string = format!("\t{}(flags) =\n\t[\n", field_name);
                let mut i = 1;

                let type_info = self.get_represented_type_info().unwrap();
                match type_info {
                    bevy::reflect::TypeInfo::Enum(enum_info) => {
                        let variants = enum_info.variant_names();
                        for variant in variants {
                            string.push_str(&format!("\t\t{} : \"{}\"\n", i, variant));
                            i += 1;
                        }
                        string.push_str("\t]");
                    }
                    _ => todo!(),
                }

                string
            }
        }
    )
    .into()
}

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(qevy_property))]
struct QevyPropertyStructAttributes {
    #[deluxe(default = "flags".to_string())]
    property_type: String,
}

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(qevy_property))]
struct QevyPropertyFieldAttributes {
    #[deluxe(default = false)]
    selected_by_default: bool,
}

fn extract_qevy_property_field_attributes(
    ast: &mut DeriveInput,
) -> deluxe::Result<HashMap<String, QevyPropertyFieldAttributes>> {
    let mut field_attrs = HashMap::new();

    if let syn::Data::Struct(s) = &mut ast.data {
        for field in s.fields.iter_mut() {
            let field_name = field.ident.as_ref().unwrap().to_string();
            let attrs: QevyPropertyFieldAttributes = deluxe::extract_attributes(field)?;
            field_attrs.insert(field_name, attrs);
        }
    }

    Ok(field_attrs)
}

fn qevy_property_derive_macro2(
    item: proc_macro2::TokenStream,
) -> deluxe::Result<proc_macro2::TokenStream> {
    // Parse
    let mut ast: DeriveInput = syn::parse2(item)?;

    // Extract struct attributes
    let QevyPropertyStructAttributes { property_type } = deluxe::extract_attributes(&mut ast)?;

    // Extract field attributes
    let field_attrs = extract_qevy_property_field_attributes(&mut ast)?;
    let (field_names, field_selected_by_defaults): (Vec<String>, Vec<bool>) = field_attrs
        .into_iter()
        .map(|(field, attrs)| (field, attrs.selected_by_default))
        .unzip();

    // define impl variables
    let ident = &ast.ident;
    let (impl_generics, type_generics, where_clause) = ast.generics.split_for_impl();

    // generate
    Ok(quote::quote!(
        impl #impl_generics QevyProperty for #ident #type_generics #where_clause {
            fn get_fgd_string(&self, field_name: &str) -> &'static str {
                let mut string = format!("\t{}({}) =\n\t[\n", field_name, #property_type);
                let mut i = 1;

                let type_info = self.get_represented_type_info().unwrap();
                match type_info {
                    bevy::reflect::TypeInfo::Enum(enum_info) => {
                        let variants = enum_info.variant_names();
                        for variant in variants {
                            string.push_str(&format!("\t\t{} : \"{}\"\n", i, variant));
                            i += 1;
                        }
                        string.push_str("\t]");
                    }
                    _ => todo!(),
                }

                Box::leak(string.into_boxed_str())
            }
        }
    ))
}

#[proc_macro_derive(QevyProperty, attributes(qevy_property))]
pub fn qevy_property_derive_macro(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    qevy_property_derive_macro2(item.into()).unwrap().into()
}
