use qevy_types::QevyEntityType;
use syn::{Attribute, DeriveInput, Meta, MetaNameValue};

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(qevy_entity))]
struct QevyEntityStructAttributes {
    #[deluxe(default = "Point".to_string())]
    entity_type: String,
}

fn extract_qevy_entity_field_comments(
    ast: &mut DeriveInput,
) -> deluxe::Result<(Vec<String>, Vec<(String, bool)>)> {
    let mut field_names = Vec::new();
    let mut field_comments = Vec::new();

    if let syn::Data::Struct(s) = &mut ast.data {
        for field in s.fields.iter_mut() {
            let field_name = field.ident.as_ref().unwrap().to_string();
            field_names.push(field_name);
            field_comments.push((get_comments(&field.attrs), is_a_base_class(&field.attrs)));
        }
    } else {
        panic!("Only structs are supported for QevyEntity derive macro");
    }

    Ok((field_names, field_comments))
}

fn is_a_base_class(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if let Meta::NameValue(MetaNameValue { path, .. }) = &attr.meta {
            if path.is_ident("QevyEntity") {
                return true;
            }
        }
    }

    false
}

pub(crate) fn qevy_entity_derive_macro2(
    item: proc_macro2::TokenStream,
) -> deluxe::Result<proc_macro2::TokenStream> {
    // Parse
    let mut ast: DeriveInput = syn::parse2(item)?;

    // Extract struct attributes
    let QevyEntityStructAttributes { entity_type } = deluxe::extract_attributes(&mut ast)?;
    // only exists to check if the entity type is valid
    let entity_type = QevyEntityType::from_short_string(entity_type.as_str())
        .expect(format!("Invalid entity type: {}", entity_type).as_str());

    let entity_type = entity_type.to_fgd_string();

    let entity_description = get_comments(&ast.attrs);

    // Extract field attributes
    let (field_names, field_attributes): (Vec<String>, Vec<(String, bool)>) =
        extract_qevy_entity_field_comments(&mut ast)?;
    let field_comments = field_attributes
        .iter()
        .map(|(comment, _)| comment)
        .collect::<Vec<&String>>();
    // enumerate through field_attributes, if it is a baseclass, get field_names with the index from the enumeration, and save it
    let base_classes = field_attributes
        .iter()
        .enumerate()
        .filter_map(|(i, (_, is_base_class))| {
            if *is_base_class {
                Some(field_names[i].clone())
            } else {
                None
            }
        })
        .collect::<Vec<String>>();
    // Join the base classes with a comma
    let base_classes = base_classes.join(", ");

    /* let base_classes =  */

    // define impl variables
    let ident = &ast.ident;
    let struct_name = ident.to_string();
    let (impl_generics, type_generics, where_clause) = ast.generics.split_for_impl();

    // Generate code
    let generated_code = quote::quote!(
        impl #impl_generics QevyEntity for #ident #type_generics #where_clause {
            fn get_export_string(
                &self,
                my_registration: &bevy::reflect::TypeRegistration,
                registry: &bevy::reflect::TypeRegistry,
                default_value: &mut Box<dyn Reflect>,
            ) -> String {
                let type_info = my_registration.type_info();
                let short_name = #struct_name;

                let field_names: Vec<&str> = vec![#(#field_names),*];
                let field_comments: Vec<&str> = vec![#(#field_comments),*];

                let base_classes = #base_classes;
                let base_class_string = if base_classes.is_empty() {
                    String::new()
                } else {
                    format!("base({})", base_classes)
                };
                let description = #entity_description;
                let entity_type = #entity_type;

                let types_string = match type_info {
                    bevy::reflect::TypeInfo::Struct(info) => {
                        let mut types_string = String::new();

                        for named_field in info.iter() {
                            // TODO: ignore base classes! as they don't implement the property trait, and don't need their own field in the fgd string!
                            let name = named_field.name();
                            let index_of_field_name = field_names.iter().position(|s| s == &name).unwrap();
                            let description = field_comments[index_of_field_name];

                            let field_type_id = named_field.type_id();
                            let field_registry = registry.get(field_type_id).unwrap();

                            let ReflectMut::Struct(mut_value) = default_value.reflect_mut() else {
                                unreachable!()
                            };
                            let property = field_registry.data::<ReflectQevyProperty>().unwrap();
                            let property = property.get(mut_value.field(name).unwrap()).unwrap();
                            let property_string = property.get_fgd_string(name, description);

                            types_string.push_str(&property_string);
                            types_string.push('\n');
                        }

                        // remove the last newline
                        types_string.pop();

                        types_string
                    }
                    _ => todo!(),
                };

                format!(
                    "{} {} = {} : \"{}\" [\n{}\n]\n",
                    entity_type, base_class_string, short_name, description, types_string
                )
            }
        }
    );

    Ok(generated_code)
}

fn get_comments(attrs: &[Attribute]) -> String {
    let mut docs = Vec::new();
    for attr in attrs {
        if let Meta::NameValue(MetaNameValue { path, value, .. }) = &attr.meta {
            if path.is_ident("doc") {
                match value {
                    syn::Expr::Lit(lit) => {
                        if let syn::Lit::Str(lit_str) = &lit.lit {
                            docs.push(lit_str.value());
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    if docs.is_empty() {
        return String::new();
    }

    // remove white space in the beginning of each line and join them
    docs.iter()
        .map(|s| s.trim())
        .collect::<Vec<&str>>()
        .join("\n")
}
