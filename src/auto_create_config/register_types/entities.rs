use std::any::{Any, TypeId};

use bevy::{prelude::*, reflect::GetTypeRegistration};

use crate::auto_create_config::QevyRegistry;

use super::QevyEntityConfig;

pub trait QevyRegisterSolidClass {
    fn register_qevy_entity<T: QevyEntityConfig + GetTypeRegistration + Any>(&mut self)
        -> &mut Self;
}

impl QevyRegisterSolidClass for App {
    fn register_qevy_entity<T: QevyEntityConfig + GetTypeRegistration + Any>(
        &mut self,
    ) -> &mut Self {
        let registry = self.world.resource_mut::<AppTypeRegistry>();
        registry.write().register::<T>();

        let mut registry = self.world.resource_mut::<QevyRegistry>();
        registry.qevy_entities.push(TypeId::of::<T>());

        self
    }
}
