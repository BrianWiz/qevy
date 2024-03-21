use bevy::prelude::*;
use qevy::auto_create_config::register_types::{
    entities::QevyRegisterSolidClass, QevyEntityConfig, QevyEntityType, ReflectQevyEntityConfig,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            qevy::MapAssetLoaderPlugin::default(),
            qevy::auto_create_config::AutoCreateConfigPlugin::default(),
        ))
        .register_qevy_entity::<AnotherSolidClass>()
        .register_qevy_entity::<TestSolidClass>()
        .register_qevy_entity::<APointClass>()
        .register_qevy_entity::<TestBaseClass>()
        .run();
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, QevyEntityConfig, Default)]
struct AnotherSolidClass;

impl QevyEntityConfig for AnotherSolidClass {
    fn get_entity_type(&self) -> &QevyEntityType {
        &QevyEntityType::Solid
    }

    fn get_base_classes(&self) -> Vec<std::any::TypeId> {
        vec![std::any::TypeId::of::<TestBaseClass>()]
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, QevyEntityConfig, Default)]
struct APointClass;

impl QevyEntityConfig for APointClass {
    fn get_entity_type(&self) -> &QevyEntityType {
        &QevyEntityType::Point
    }

    fn get_base_classes(&self) -> Vec<std::any::TypeId> {
        vec![std::any::TypeId::of::<TestBaseClass>()]
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, QevyEntityConfig, Default)]
struct TestSolidClass;

impl QevyEntityConfig for TestSolidClass {
    fn get_entity_type(&self) -> &QevyEntityType {
        &QevyEntityType::Solid
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, QevyEntityConfig, Default)]
struct TestBaseClass;

impl QevyEntityConfig for TestBaseClass {
    fn get_entity_type(&self) -> &QevyEntityType {
        &QevyEntityType::Base
    }
}
