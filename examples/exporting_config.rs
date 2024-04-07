use bevy::prelude::*;
use qevy::auto_create_config::register_types::{
    entities::QevyRegisterSolidClass,
    properties::{QevyProperty, ReflectQevyProperty},
    QevyEntity, ReflectQevyEntity,
};
use qevy_derive::QevyEntity;
use qevy_types::QevyEntityType;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            qevy::MapAssetLoaderPlugin::default(),
            qevy::auto_create_config::AutoCreateConfigPlugin::default(),
        ))
        .register_qevy_entity::<TestSolidClass>()
        .register_qevy_entity::<APointClass>()
        .register_qevy_entity::<TestBaseClass>()
        .register_type::<EnumTestFlag>()
        .register_type::<EnumTestChoices>()
        .run();
}

#[derive(Reflect, QevyEntity)]
#[reflect(QevyEntity, Default)]
#[qevy_entity(
    entity_type = "Point",
    desc = "This is a simple testing class, showcasing the different property types."
)]
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
    test_flag: EnumTestFlag,
    test_choices: EnumTestChoices,
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
            test_flag: EnumTestFlag::EnumVariantTest,
            test_choices: EnumTestChoices::EnumVariantTest,
        }
    }
}

#[derive(Reflect, Default, QevyProperty)]
#[reflect(QevyProperty, Default)]
#[qevy_property(property_type = "flags")]
enum EnumTestFlag {
    #[default]
    #[qevy_property(selected_by_default = true)]
    Test,
    #[qevy_property(selected_by_default = false)]
    EnumVariantTest,
    TestTestTest,
}

#[derive(Reflect, Default, QevyProperty)]
#[reflect(QevyProperty, Default)]
#[qevy_property(property_type = "choices")]
enum EnumTestChoices {
    #[default]
    #[qevy_property(selected_by_default = true)]
    Test,
    #[qevy_property(key_override = "VariantTest!!!!")]
    EnumVariantTest,
    AnotherEnumVariant,
}

#[derive(Reflect, Default)]
#[reflect(QevyEntity, Default)]
struct TestSolidClass;

impl QevyEntity for TestSolidClass {
    fn get_entity_type(&self) -> QevyEntityType {
        QevyEntityType::Solid
    }
}

#[derive(Reflect, Default)]
#[reflect(QevyEntity, Default)]
struct TestBaseClass;

impl QevyEntity for TestBaseClass {
    fn get_entity_type(&self) -> QevyEntityType {
        QevyEntityType::Base
    }
}
