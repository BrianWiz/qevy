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

    fn get_export_string(
        &self,
        my_registration: &TypeRegistration,
        registry: &TypeRegistry,
        default_value: &mut Box<dyn Reflect>,
    ) -> String;

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
