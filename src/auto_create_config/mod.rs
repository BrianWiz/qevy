use std::{any::TypeId, path::PathBuf};

use bevy::prelude::*;

use self::export::create_config;
mod export;
pub mod register_types;

#[derive(Resource, Clone, Default)]
pub(crate) struct QevyRegistry {
    pub(crate) qevy_entities: Vec<TypeId>,
    pub(crate) base_classes: Vec<TypeId>,
}

#[derive(Resource, Reflect, Clone)]
#[reflect(Resource)]
pub(crate) struct AutoCreateConfigSettings {
    pub save_path: PathBuf,
}

impl Default for AutoCreateConfigSettings {
    fn default() -> Self {
        AutoCreateConfigSettings {
            save_path: PathBuf::from("game_entities.fgd"),
        }
    }
}

pub struct AutoCreateConfigPlugin {
    pub(crate) settings: AutoCreateConfigSettings,
}

impl Default for AutoCreateConfigPlugin {
    fn default() -> Self {
        AutoCreateConfigPlugin {
            settings: AutoCreateConfigSettings::default(),
        }
    }
}

impl AutoCreateConfigPlugin {
    pub fn new(save_path: PathBuf) -> Self {
        AutoCreateConfigPlugin {
            settings: AutoCreateConfigSettings { save_path },
        }
    }
}

impl Plugin for AutoCreateConfigPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AutoCreateConfigSettings>()
            .init_resource::<QevyRegistry>()
            .register_asset_root()
            .insert_resource(self.settings.clone())
            .add_systems(Startup, create_config);
    }
}

trait RegistryExportApp {
    fn register_asset_root(&mut self) -> &mut Self;
}
impl RegistryExportApp for App {
    fn register_asset_root(&mut self) -> &mut Self {
        let asset_plugin = get_asset_plugin(self);
        let path_str = asset_plugin.file_path.clone();
        let path = PathBuf::from(path_str);
        self.insert_resource(AssetRoot(path))
    }
}

fn get_asset_plugin(app: &App) -> &AssetPlugin {
    let asset_plugins: Vec<&AssetPlugin> = app.get_added_plugins();
    asset_plugins.into_iter().next().expect(ASSET_ERROR)
}

const ASSET_ERROR: &str = "Qevy requires access to the Bevy asset plugin. \
    Please add `AutoCreateConfigPlugin` after `AssetPlugin`, which is commonly added as part of the `DefaultPlugins`";

#[derive(Debug, Clone, PartialEq, Eq, Hash, Resource)]
pub(crate) struct AssetRoot(pub(crate) PathBuf);
