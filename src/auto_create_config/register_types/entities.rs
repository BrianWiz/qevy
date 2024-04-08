use std::any::{Any, TypeId};

use bevy::{prelude::*, reflect::GetTypeRegistration};
use qevy_types::QevyEntityType;

use crate::auto_create_config::QevyRegistry;

use super::QevyEntity;

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
