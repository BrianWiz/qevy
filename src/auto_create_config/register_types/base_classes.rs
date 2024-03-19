use std::any::{Any, TypeId};

use bevy::{prelude::*, reflect::GetTypeRegistration};

use crate::auto_create_config::QevyRegistry;

use super::QevyEntityConfig;

pub trait QevyRegisterBaseClass {
    fn register_qevy_base_class<T: QevyEntityConfig + GetTypeRegistration + Any>(
        &mut self,
    ) -> &mut Self;
}

impl QevyRegisterBaseClass for App {
    fn register_qevy_base_class<T: QevyEntityConfig + GetTypeRegistration + Any>(
        &mut self,
    ) -> &mut Self {
        let registry = self.world.resource_mut::<AppTypeRegistry>();
        registry.write().register::<T>();

        let mut registry = self.world.resource_mut::<QevyRegistry>();
        registry.base_classes.push(TypeId::of::<T>());

        self
    }
}
