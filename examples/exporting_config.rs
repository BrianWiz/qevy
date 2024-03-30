use bevy::prelude::*;
use qevy::auto_create_config::register_types::{
    entities::QevyRegisterSolidClass, properties::ReflectQevyProperty, QevyEntityConfig,
    QevyEntityType, ReflectQevyEntityConfig,
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
struct APointClass {
    test_string: String,
    test_usize: usize,
    test_bool: bool,
    test_f32: f32,
    test_f64: f64,
    test_i32: i32,
    test_i64: i64,
    test_u32: u32,
    test_u64: u64,
    test_enum: EnumTest,
}

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
struct AnotherSolidClass;

#[derive(Reflect, Default)]
#[reflect(QevyProperty, Default)]
enum EnumTest {
    #[default]
    Test,
    EnumVariantTest,
}

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
