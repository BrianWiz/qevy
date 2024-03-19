use std::any::TypeId;

use bevy::reflect::{reflect_trait, TypeRegistration, TypeRegistry};

pub mod base_classes;
pub mod entities;

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
pub trait QevyEntityConfig {
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
    ) -> String {
        let short_name = my_registration.type_info().type_path_table().short_path();

        let base_class_string = match self.get_entity_type() {
            QevyEntityType::Base => String::new(), // Base classes don't have base classes. I think?
            _ => format!("base({})", self.get_base_classes_fgd_string(registry)),
        };
        let description = self.get_description();
        let entity_type = self.get_entity_type().to_fgd_string();

        format!(
            "{} {} = {} : \"{}\" []",
            entity_type, base_class_string, short_name, description
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
