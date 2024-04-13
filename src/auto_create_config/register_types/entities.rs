use std::any::{Any, TypeId};

use bevy::{
    prelude::*,
    reflect::{GetTypeRegistration, TypeRegistration, TypeRegistry},
};

use crate::auto_create_config::QevyRegistry;

#[reflect_trait]
pub trait QevyEntity: Reflect {
    fn get_export_string(
        &self,
        my_registration: &TypeRegistration,
        registry: &TypeRegistry,
        default_value: &mut Box<dyn Reflect>,
    ) -> String;
}

pub trait QevyRegisterSolidClass {
    fn register_qevy_entity<T: QevyEntity + GetTypeRegistration + Any + Default>(
        &mut self,
    ) -> &mut Self;
}

impl QevyRegisterSolidClass for App {
    fn register_qevy_entity<T: QevyEntity + GetTypeRegistration + Any + Default>(
        &mut self,
    ) -> &mut Self {
        let registry = self.world.resource_mut::<AppTypeRegistry>();
        registry.write().register::<T>();

        let mut registry = self.world.resource_mut::<QevyRegistry>();
        registry.qevy_entities.push(TypeId::of::<T>());

        self
    }
}
