use bevy::prelude::*;
use qevy::auto_create_config::register_types::{
    entities::QevyRegisterSolidClass,
    properties::{QevyProperty, ReflectQevyProperty},
    QevyEntityConfig, QevyEntityType, ReflectQevyEntityConfig,
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
        .register_type::<EnumTestFlag>()
        .run();
}

#[derive(Reflect)]
#[reflect(QevyEntityConfig, Default)]
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
    test_enum: EnumTestFlag,
}

impl Default for APointClass {
    fn default() -> Self {
        Self {
            test_string: "HELLO WORLD!".to_string(),
            test_usize: 69,
            test_bool: true,
            test_f32: 69.420,
            test_f64: 420.69,
            test_i32: Default::default(),
            test_i64: Default::default(),
            test_u32: Default::default(),
            test_u64: Default::default(),
            test_enum: EnumTestFlag::EnumVariantTest,
        }
    }
}

impl QevyEntityConfig for APointClass {
    fn get_entity_type(&self) -> &QevyEntityType {
        &QevyEntityType::Point
    }

    fn get_base_classes(&self) -> Vec<std::any::TypeId> {
        vec![std::any::TypeId::of::<TestBaseClass>()]
    }
}

#[derive(Reflect, Default)]
#[reflect(QevyEntityConfig, Default)]
struct AnotherSolidClass;

impl QevyEntityConfig for AnotherSolidClass {
    fn get_entity_type(&self) -> &QevyEntityType {
        &QevyEntityType::Solid
    }

    fn get_base_classes(&self) -> Vec<std::any::TypeId> {
        vec![std::any::TypeId::of::<TestBaseClass>()]
    }
}

#[derive(Reflect, Default, QevyProperty)]
#[reflect(QevyProperty, Default)]
#[qevy_property(property_type = "flags")] // TODO: Flags and Choices
enum EnumTestFlag {
    #[default]
    Test,
    EnumVariantTest,
}

#[derive(Reflect, Default)]
#[reflect(QevyEntityConfig, Default)]
struct TestSolidClass;

impl QevyEntityConfig for TestSolidClass {
    fn get_entity_type(&self) -> &QevyEntityType {
        &QevyEntityType::Solid
    }
}

#[derive(Reflect, Default)]
#[reflect(QevyEntityConfig, Default)]
struct TestBaseClass;

impl QevyEntityConfig for TestBaseClass {
    fn get_entity_type(&self) -> &QevyEntityType {
        &QevyEntityType::Base
    }
}
