use bevy::reflect::{reflect_trait, Reflect, TypeRegistration, TypeRegistry};

pub mod entities;
pub mod properties;

#[reflect_trait]
pub trait QevyEntity: Reflect {
    fn get_export_string(
        &self,
        my_registration: &TypeRegistration,
        registry: &TypeRegistry,
        default_value: &mut Box<dyn Reflect>,
    ) -> String;
}
