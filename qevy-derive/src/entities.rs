use qevy_types::QevyEntityType;
use syn::{Attribute, DeriveInput, Meta, MetaNameValue};

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(qevy_entity))]
struct QevyEntityStructAttributes {
    #[deluxe(default = "Point".to_string())]
    entity_type: String,
    #[deluxe(default = None)]
    entity_name: Option<String>,
    // (path, frame, skin, scale)
    #[deluxe(default = None)]
    model: Option<(String, Option<u32>, Option<u32>, Option<u32>)>,
    // -x,-y,-z,+x,+y,+z
    #[deluxe(default = None)]
    size: Option<(u32, u32, u32, u32, u32, u32)>,
    // (r, g, b), 0-255
    #[deluxe(default = None)]
    color: Option<(u32, u32, u32)>,
}

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(qevy_entity))]
struct QevyEntityFieldAttributes {
    #[deluxe(default = None)]
    base_class: Option<String>,
}

fn extract_qevy_entity_field_comments(
    ast: &mut DeriveInput,
) -> deluxe::Result<(Vec<String>, Vec<(String, Option<String>)>)> {
    let mut field_names = Vec::new();
    let mut field_attributes = Vec::new();

    if let syn::Data::Struct(s) = &mut ast.data {
        for field in s.fields.iter_mut() {
            let field_name = field.ident.as_ref().unwrap().to_string();
            field_names.push(field_name);

            let attrs: QevyEntityFieldAttributes = deluxe::extract_attributes(field)?;

            field_attributes.push((get_comments(&field.attrs), attrs.base_class));
        }
    } else {
        panic!("Only structs are supported for QevyEntity derive macro");
    }

    Ok((field_names, field_attributes))
}

pub(crate) fn qevy_entity_derive_macro2(
    item: proc_macro2::TokenStream,
) -> deluxe::Result<proc_macro2::TokenStream> {
    // Parse
    let mut ast: DeriveInput = syn::parse2(item)?;

    // Extract struct attributes
    let QevyEntityStructAttributes {
        entity_type,
        entity_name,
        model,
        size,
        color,
    } = deluxe::extract_attributes(&mut ast)?;

    let model_string = model
        .map(|(path, frame, skin, scale)| {
            format!(
                "model({{\n\t\"path\" : \"{}\",\n\t\"frame\" : {},\n\t\"skin\" : {},\n\t\"scale\" : {}\n}})",
                path,
                frame.unwrap_or(0),
                skin.unwrap_or(0),
                scale.unwrap_or(32)
            )
        })
        .unwrap_or_else(|| String::new());

    let entity_size_string = size
        .map(|(min_x, min_y, min_z, max_x, max_y, max_z)| {
            format!(
                "size(-{} -{} -{}, {} {} {})",
                min_x, min_y, min_z, max_x, max_y, max_z
            )
        })
        .unwrap_or_else(|| String::new());

    let color_string = color
        .map(|(r, g, b)| format!("color({} {} {})", r, g, b))
        .unwrap_or_else(|| String::new());

    let entity_type = QevyEntityType::from_short_string(entity_type.as_str())
        .expect(format!("Invalid entity type: {}", entity_type).as_str());

    let entity_type = entity_type.to_fgd_string();

    let entity_description = get_comments(&ast.attrs);

    // Extract field attributes
    let (field_names, field_attributes): (Vec<String>, Vec<(String, Option<String>)>) =
        extract_qevy_entity_field_comments(&mut ast)?;
    let field_comments = field_attributes
        .iter()
        .map(|(comment, ..)| comment)
        .collect::<Vec<&String>>();
    // enumerate through field_attributes, if it is a baseclass, get field_names with the index from the enumeration, and save it
    let base_classes = field_attributes
        .iter()
        .enumerate()
        .filter_map(|(i, (_, base_class_name))| {
            base_class_name.as_ref().map(|base_class_name| {
                if base_class_name.is_empty() {
                    panic!("Base class name cannot be empty");
                }
                (field_names[i].clone(), base_class_name.clone())
            })
        })
        .collect::<Vec<(String, String)>>();
    let base_classes_field_names = base_classes.iter().map(|(field_name, _)| field_name);
    let base_classes_base_class_names = base_classes
        .iter()
        .map(|(_, base_class_name)| base_class_name);

    // define impl variables
    let ident = &ast.ident;
    // if the entity_name is not set, use the struct name
    let struct_name = entity_name.unwrap_or_else(|| ident.to_string());
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

                let size_string = #entity_size_string;
                let color_string = #color_string;
                let model_string: &str = #model_string;

                let base_classes_field_names: Vec<&str> = vec![#(#base_classes_field_names),*];
                let base_classes_base_class_names: Vec<&str> = vec![#(#base_classes_base_class_names),*];
                let joined_base_classes = base_classes_base_class_names.join(", ");
                let base_class_string = if joined_base_classes.is_empty() {
                    String::new()
                } else {
                    format!("base({})", joined_base_classes)
                };
                let description = #entity_description;
                let entity_type = #entity_type;

                let types_string = match type_info {
                    bevy::reflect::TypeInfo::Struct(info) => {
                        let mut types_string = String::new();

                        for named_field in info.iter() {
                            let name = named_field.name();

                            // Ignore base classes, as they don't need their own field in the fgd
                            if base_classes_field_names.iter().any(|&base_class| base_class == name) {
                                continue;
                            }

                            let index_of_field_name = field_names.iter().position(|s| s == &name).expect(format!("Field name not found: {}", name).as_str());
                            let description = field_comments[index_of_field_name];

                            let field_type_id = named_field.type_id();
                            let field_registry = registry.get(field_type_id).expect(format!("Field type not found: {}", name).as_str());

                            let ReflectMut::Struct(mut_value) = default_value.reflect_mut() else {
                                unreachable!("Default value is not a struct");
                            };
                            let property = field_registry.data::<ReflectQevyProperty>().expect(format!("Field type does not implement ReflectQevyProperty: {}", name).as_str());
                            let property = property.get(mut_value.field(name).unwrap().try_as_reflect().unwrap()).expect(format!("Field not found: {}", name).as_str());
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
                    "{} {} {} {} {} = {} : \"{}\" [\n{}\n]\n",
                    entity_type, base_class_string, size_string, color_string, model_string, short_name, description, types_string
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
