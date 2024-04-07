use std::any::TypeId;

use bevy::reflect::{reflect_trait, Reflect, ReflectMut, TypeRegistration, TypeRegistry};
use qevy_types::QevyEntityType;

use crate::auto_create_config::register_types::properties::ReflectQevyProperty;

pub mod entities;
pub mod properties;

#[reflect_trait]
pub trait QevyEntity: Reflect {
    fn get_base_classes(&self) -> Vec<TypeId> {
        vec![] // TODO: fill this with structs that implement QevyEntityConfig:Base
    }

    fn get_description(&self) -> &str {
        ""
    }

    fn get_entity_type(&self) -> QevyEntityType; // TODO: Derive

    fn get_export_string(
        &self,
        my_registration: &TypeRegistration,
        registry: &TypeRegistry,
        default_value: &mut Box<dyn Reflect>,
    ) -> String {
        let type_info = my_registration.type_info();
        let short_name = type_info.type_path_table().short_path();

        let base_class_string = match self.get_entity_type() {
            QevyEntityType::Base => String::new(), // Base classes don't have base classes. I think?
            _ => format!("base({})", self.get_base_classes_fgd_string(registry)),
        };
        let description = self.get_description();
        let entity_type = self.get_entity_type();
        let entity_type = entity_type.to_fgd_string();

        let types_string = match type_info {
            bevy::reflect::TypeInfo::Struct(info) => {
                let mut types_string = String::new();

                for named_field in info.iter() {
                    let name = named_field.name();
                    let field_type_id = named_field.type_id();
                    let field_registry = registry.get(field_type_id).unwrap();

                    let ReflectMut::Struct(mut_value) = default_value.reflect_mut() else {
                        unreachable!()
                    };
                    let property = field_registry.data::<ReflectQevyProperty>().unwrap();
                    let property = property.get(mut_value.field(name).unwrap()).unwrap();
                    let property_string = property.get_fgd_string(name, "");

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
