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
    #[deluxe(default = None)]
    key_override: Option<String>,
}

fn extract_qevy_property_field_attributes(
    ast: &mut DeriveInput,
) -> deluxe::Result<(Vec<String>, Vec<QevyPropertyFieldAttributes>)> {
    let mut field_names = Vec::new();
    let mut field_attrs = Vec::new();

    if let syn::Data::Enum(e) = &mut ast.data {
        for variant in e.variants.iter_mut() {
            let variant_name = variant.ident.to_string();
            let attrs: QevyPropertyFieldAttributes = deluxe::extract_attributes(variant)?;
            field_names.push(variant_name);
            field_attrs.push(attrs);
        }
    }

    Ok((field_names, field_attrs))
}

fn qevy_property_derive_macro2(
    item: proc_macro2::TokenStream,
) -> deluxe::Result<proc_macro2::TokenStream> {
    // Parse
    let mut ast: DeriveInput = syn::parse2(item)?;

    // Extract struct attributes
    let QevyPropertyStructAttributes { property_type } = deluxe::extract_attributes(&mut ast)?;
    let ident = &ast.ident;
    let ident_name = ident.to_string();

    // Extract field attributes
    let (field_names, field_attrs): (Vec<String>, Vec<QevyPropertyFieldAttributes>) =
        extract_qevy_property_field_attributes(&mut ast)?;

    let formatted_field_strings = match property_type.as_str() {
        "flags" => {
            let mut formatted_field_strings = String::new();
            formatted_field_strings.push_str(" =\n\t[\n");

            formatted_field_strings.push_str(
                field_names
                    .iter()
                    .enumerate()
                    .map(|(i, field)| {
                        let selected_by_default = if field_attrs[i].selected_by_default {
                            "1"
                        } else {
                            "0"
                        };
                        format!(
                            "\t\t{} : \"{}\" : {}\n",
                            2_i32.pow(i as u32),
                            field,
                            selected_by_default
                        )
                    })
                    .collect::<Vec<String>>()
                    .join("")
                    .as_str(),
            );

            formatted_field_strings
        }
        "choices" => {
            let selected_index = field_attrs
                .iter()
                .position(|selected| selected.selected_by_default)
                .expect("No default selected choice");
            let selected_key = match field_attrs[selected_index].key_override.as_ref() {
                Some(key) => format!("\"{}\"", key),
                None => format!("{}", selected_index),
            };

            let mut formatted_field_strings = String::new();
            formatted_field_strings.push_str(
                format!(
                    " : \"{}\" : {} =\n\t[\n",
                    ident_name, // TODO: Somehow replace with the field name of the original struct.
                    selected_key
                )
                .as_str(),
            );

            for (index, field_name) in field_names.iter().enumerate() {
                let attributes = &field_attrs[index];
                // Key is optional, if not provided, use the index
                let key = match attributes.key_override.as_ref() {
                    Some(key) => format!("\"{}\"", key),
                    None => format!("{}", index),
                };

                formatted_field_strings.push_str(&format!("\t\t{} : \"{}\"\n", key, field_name));
            }

            formatted_field_strings
        }
        _ => panic!("Unsupported property type: {}", property_type),
    };

    // define impl variables
    let ident = &ast.ident;
    let (impl_generics, type_generics, where_clause) = ast.generics.split_for_impl();

    // generate
    let generated_code = quote::quote!(
        impl #impl_generics QevyProperty for #ident #type_generics #where_clause {
            fn get_fgd_string(&self, field_name: &str, field_description: &str) -> &'static str {
                let mut string = format!("\t{}({})", field_name, #property_type);

                string.push_str(#formatted_field_strings);

                string.push_str("\t]");

                Box::leak(string.into_boxed_str())
            }
        }
    );

    Ok(generated_code)
}

#[proc_macro_derive(QevyProperty, attributes(qevy_property))]
pub fn qevy_property_derive_macro(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    qevy_property_derive_macro2(item.into()).unwrap().into()
}
