use std::collections::HashMap;

use syn::DeriveInput;

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

    if let syn::Data::Enum(e) = &mut ast.data {
        for variant in e.variants.iter_mut() {
            let variant_name = variant.ident.to_string();
            let attrs: QevyPropertyFieldAttributes = deluxe::extract_attributes(variant)?;
            field_attrs.insert(variant_name, attrs);
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

    let formatted_field_strings = field_names
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let selected_by_default = if field_selected_by_defaults[i] {
                "1"
            } else {
                "0"
            };
            format!("\t\t{} : \"{}\" : {}\n", i + 1, field, selected_by_default)
            // i + 1 if you want to start counting from 1
        })
        .collect::<Vec<String>>()
        .join(""); // Join all strings into a single string

    // define impl variables
    let ident = &ast.ident;
    let (impl_generics, type_generics, where_clause) = ast.generics.split_for_impl();

    let generated_code = match property_type.as_str() {
        "flags" => {
            quote::quote!(
                impl #impl_generics QevyProperty for #ident #type_generics #where_clause {
                    fn get_fgd_string(&self, field_name: &str) -> &'static str {
                        let mut string = format!("\t{}({}) =\n\t[\n", field_name, #property_type);

                        string.push_str(#formatted_field_strings);

                        string.push_str("\t]");

                        Box::leak(string.into_boxed_str())
                    }
                }
            )
        }
        _ => panic!("Unsupported property type: {}", property_type),
    };

    // generate
    Ok(generated_code)
}

#[proc_macro_derive(QevyProperty, attributes(qevy_property))]
pub fn qevy_property_derive_macro(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    qevy_property_derive_macro2(item.into()).unwrap().into()
}
