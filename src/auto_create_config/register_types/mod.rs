use std::any::TypeId;

use bevy::reflect::{reflect_trait, Reflect, TypeRegistration, TypeRegistry};

pub mod entities;
pub mod properties;

#[derive(Debug, PartialEq)]
pub enum QevyEntityType {
    Base,
    Solid,
    Point,
}

impl QevyEntityType {
    pub fn to_fgd_string(&self) -> &str {
        match self {
            QevyEntityType::Base => "@BaseClass",
            QevyEntityType::Solid => "@SolidClass",
            QevyEntityType::Point => "@PointClass",
        }
    }
}

#[reflect_trait]
pub trait QevyEntityConfig: Reflect {
    fn get_base_classes(&self) -> Vec<TypeId> {
        vec![]
    }

    fn get_description(&self) -> &str {
        ""
    }

    fn get_entity_type(&self) -> &QevyEntityType;

    fn get_export_string(
        &self,
        my_registration: &TypeRegistration,
        registry: &TypeRegistry,
        default_value: &Box<dyn Reflect>,
    ) -> String {
        let type_info = my_registration.type_info();
        let short_name = type_info.type_path_table().short_path();

        let base_class_string = match self.get_entity_type() {
            QevyEntityType::Base => String::new(), // Base classes don't have base classes. I think?
            _ => format!("base({})", self.get_base_classes_fgd_string(registry)),
        };
        let description = self.get_description();
        let entity_type = self.get_entity_type().to_fgd_string();

        let types_string = match type_info {
            bevy::reflect::TypeInfo::Struct(info) => {
                let mut types_string = String::new();
                types_string.push('\n');

                for named_field in info.iter() {
                    let field_type_id = named_field.type_id();
                    let name = named_field.name();
                    let type_path = named_field.type_path_table().short_path();
                    let fgd_type = match convert_types_to_fgd(type_path) {
                        Ok(type_path) => type_path,
                        Err(_) => "",
                    };

                    types_string.push_str(&format!(
                        "{}({}) : \"\" : {}\n",
                        name,
                        fgd_type,
                        get_default_value_for_field(default_value, name, fgd_type)
                    ));
                }

                // remove the last newline
                types_string.pop();

                types_string
            }
            bevy::reflect::TypeInfo::TupleStruct(_) => todo!(),
            bevy::reflect::TypeInfo::Tuple(_) => todo!(),
            bevy::reflect::TypeInfo::List(_) => todo!(),
            bevy::reflect::TypeInfo::Array(_) => todo!(),
            bevy::reflect::TypeInfo::Map(_) => todo!(),
            bevy::reflect::TypeInfo::Enum(_) => todo!(),
            bevy::reflect::TypeInfo::Value(_) => todo!(),
        };

        format!(
            "{} {} = {} : \"{}\" [{}]",
            entity_type, base_class_string, short_name, description, types_string
        )
    }

    fn get_base_classes_fgd_string(&self, registry: &TypeRegistry) -> String {
        let base_classes = self.get_base_classes();

        if base_classes.is_empty() {
            return String::new();
        }

        let mut base_classes_string = String::new();
        for base_class_type_id in base_classes {
            let schema = registry.get(base_class_type_id).unwrap();
            let type_info = schema.type_info();
            let binding = type_info.type_path_table();
            let short_name = binding.short_path();

            base_classes_string.push_str(short_name);
        }

        base_classes_string
    }
}

struct TypeNotSupported(String);

fn convert_types_to_fgd(short_type: &str) -> Result<&str, TypeNotSupported> {
    match short_type {
        "String" => Ok("string"),
        "usize" | "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" => Ok("integer"),
        "f32" | "f64" => Ok("float"),
        "bool" => Ok("boolean"),
        _ => Err(TypeNotSupported(short_type.to_string())),
    }
}

fn get_default_value_for_field(
    default_value: &Box<dyn Reflect>,
    field_name: &str,
    field_type: &str,
) -> String {
    match default_value.reflect_ref() {
        bevy::reflect::ReflectRef::Struct(default_struct) => {
            if let Some(field) = default_struct.field(field_name) {
                println!("Field Name: {}, Type: {}", field_name, field_type);
                actually_get_default_value_for_field(field, field_type)
            } else {
                panic!("Field not found: {}", field_name);
            }
        }
        bevy::reflect::ReflectRef::TupleStruct(_) => todo!(),
        bevy::reflect::ReflectRef::Tuple(_) => todo!(),
        bevy::reflect::ReflectRef::List(_) => todo!(),
        bevy::reflect::ReflectRef::Array(_) => todo!(),
        bevy::reflect::ReflectRef::Map(_) => todo!(),
        bevy::reflect::ReflectRef::Enum(_) => todo!(),
        bevy::reflect::ReflectRef::Value(_) => todo!(),
    }
}

fn actually_get_default_value_for_field(field: &dyn Reflect, field_type: &str) -> String {
    println!("Field: {:?}", field);

    match field_type {
        "string" => {
            let value = field.downcast_ref::<String>().unwrap();
            format!("\"{}\"", value)
        }
        "integer" => {
            if let Some(value) = field.downcast_ref::<usize>() {
                return value.to_string();
            }

            if let Some(value) = field.downcast_ref::<u8>() {
                return value.to_string();
            }

            if let Some(value) = field.downcast_ref::<u16>() {
                return value.to_string();
            }

            if let Some(value) = field.downcast_ref::<u32>() {
                return value.to_string();
            }

            if let Some(value) = field.downcast_ref::<u64>() {
                return value.to_string();
            }

            if let Some(value) = field.downcast_ref::<i8>() {
                return value.to_string();
            }

            if let Some(value) = field.downcast_ref::<i16>() {
                return value.to_string();
            }

            if let Some(value) = field.downcast_ref::<i32>() {
                return value.to_string();
            }

            if let Some(value) = field.downcast_ref::<i64>() {
                return value.to_string();
            }

            panic!("Field type not supported: {}", field_type);
        }
        "float" => {
            if let Some(value) = field.downcast_ref::<f32>() {
                return format!("\"{}\"", value);
            }

            if let Some(value) = field.downcast_ref::<f64>() {
                return format!("\"{}\"", value);
            }

            panic!("Field type not supported: {}", field_type);
        }
        "boolean" => {
            let value = field.downcast_ref::<bool>().unwrap();
            
            match value {
                true => "yes".to_string(),
                false => "no".to_string(),
            }
        }
        _ => panic!("Field type not supported: {}", field_type),
    }
}
