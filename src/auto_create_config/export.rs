use std::{fs::File, io::Write, path::Path};

use bevy::{
    prelude::*,
    reflect::{TypeRegistration, TypeRegistry},
};

use super::{
    register_types::ReflectQevyEntityConfig, AssetRoot, AutoCreateConfigSettings, QevyRegistry,
};

pub(crate) fn create_config(world: &mut World) {
    let config = world.resource::<AutoCreateConfigSettings>();
    let qevy_registry = world.resource::<QevyRegistry>();
    let asset_root = world.resource::<AssetRoot>();
    let types = world.resource::<AppTypeRegistry>();
    let types = types.read();

    let registry_save_path = Path::join(&asset_root.0, &config.save_path);
    let mut writer = File::create(registry_save_path).expect("could not create file");

    // Base classes
    let qevy_base_classes_registrations: Vec<_> = qevy_registry
        .base_classes
        .iter()
        .filter_map(|base_class_type| types.get(*base_class_type))
        .collect();

    for qevy_base_class_reg in qevy_base_classes_registrations {
        let config_string = type_reg_to_export_string(qevy_base_class_reg, &*types);

        writer
            .write_all(config_string.as_bytes())
            .expect("could not write to file");
    }

    // Solid and Point classes
    let qevy_entities_registrations: Vec<_> = qevy_registry
        .qevy_entities
        .iter()
        .filter_map(|qevy_entity_type| types.get(*qevy_entity_type))
        .collect();

    for qevy_entity_reg in qevy_entities_registrations {
        let config_string = type_reg_to_export_string(qevy_entity_reg, &*types);

        writer
            .write_all(config_string.as_bytes())
            .expect("could not write to file");
    }
}

fn type_reg_to_export_string(type_reg: &TypeRegistration, registry: &TypeRegistry) -> String {
    let reflect_default = type_reg.data::<ReflectDefault>().unwrap();
    let value: Box<dyn Reflect> = reflect_default.default();
    if let Some(reflect_entity_config) = type_reg.data::<ReflectQevyEntityConfig>() {
        let entity_config = reflect_entity_config.get(&*value).unwrap();

        return entity_config.get_export_string(type_reg, registry) + "\n";
    }

    let short_name = type_reg.type_info().type_path_table().short_path();
    panic!("No ReflectQevyEntityConfig for type: {}\nThat could happen because you didn't add \"#[reflect[QevyEntityConfig)]\" to the component!", short_name);
}
