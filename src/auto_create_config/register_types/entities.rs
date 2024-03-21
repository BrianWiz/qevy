use std::any::{Any, TypeId};

use bevy::{prelude::*, reflect::GetTypeRegistration};

use crate::auto_create_config::QevyRegistry;

use super::{QevyEntityConfig, QevyEntityType};

pub trait QevyRegisterSolidClass {
    fn register_qevy_entity<T: QevyEntityConfig + GetTypeRegistration + Any + Default>(
        &mut self,
    ) -> &mut Self;
}

impl QevyRegisterSolidClass for App {
    fn register_qevy_entity<T: QevyEntityConfig + GetTypeRegistration + Any + Default>(
        &mut self,
    ) -> &mut Self {
        let registry = self.world.resource_mut::<AppTypeRegistry>();
        registry.write().register::<T>();

        let mut registry = self.world.resource_mut::<QevyRegistry>();

        let t = T::default();
        match t.get_entity_type() {
            QevyEntityType::Base => {
                registry.base_classes.push(TypeId::of::<T>());
            }
            _ => {
                registry.qevy_entities.push(TypeId::of::<T>());
            }
        }

        self
    }
}
