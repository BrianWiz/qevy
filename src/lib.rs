use bevy::asset::io::Reader;
use bevy::asset::AsyncReadExt;
use bevy::asset::{AssetLoader, BoxedFuture, Handle, LoadContext};
use bevy::ecs::system::SystemId;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use std::collections::BTreeMap;
use thiserror::Error;
use tracing::info;

mod build;
pub mod components;
pub mod conversions;
pub mod gameplay_systems;
pub mod load;

#[derive(Debug, Asset, TypePath)]
pub struct MapAsset {
    geomap: Option<shambler::GeoMap>,
    texture_sizes: BTreeMap<String, (u32, u32)>,
    material_handles: BTreeMap<String, Handle<StandardMaterial>>,
}

impl MapAsset {
    pub fn get_texture_names_with_size(&self) -> BTreeMap<&str, (u32, u32)> {
        let mut names: BTreeMap<&str, (u32, u32)> = BTreeMap::new();
        for (texture_name, (width, height)) in &self.texture_sizes {
            names.insert(texture_name.as_str(), (*width, *height));
        }
        names
    }
}

#[derive(Debug, Error)]
pub enum MapAssetLoaderError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Default)]
pub struct MapAssetLoader;

impl AssetLoader for MapAssetLoader {
    type Asset = MapAsset;
    type Settings = ();
    type Error = MapAssetLoaderError;
    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        load::load(reader, load_context, false)
    }

    fn extensions(&self) -> &[&str] {
        &["map"]
    }
}

#[derive(Default)]
pub struct HeadlessMapAssetLoader;

impl AssetLoader for HeadlessMapAssetLoader {
    type Asset = MapAsset;
    type Settings = ();
    type Error = MapAssetLoaderError;
    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        load::load(reader, load_context, true)
    }

    fn extensions(&self) -> &[&str] {
        load::extensions()
    }
}

#[derive(Event)]
pub struct PostBuildMapEvent {
    pub map: Entity,
}

#[derive(Default, Resource)]
pub struct PostMapBuildHook {
    pub system: Option<SystemId>,
}

#[derive(Default)]
pub struct MapAssetLoaderPlugin {
    /// If true, the plugin will not add meshes, only colliders
    pub headless: bool,
}

impl Plugin for MapAssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<MapAsset>()
            .init_resource::<PostMapBuildHook>()
            .add_event::<components::TriggeredEvent>()
            .add_event::<PostBuildMapEvent>()
            .add_event::<build::SpawnMeshEvent>();

        if self.headless {
            info!("Using headless map loader. Only colliders will be added.");
            app.add_systems(PreUpdate, load::handle_loaded_map_system);
            app.init_asset_loader::<HeadlessMapAssetLoader>();
        } else {
            app.add_systems(
                PreUpdate,
                (load::handle_loaded_map_system, build::mesh_spawn_system).chain(),
            );
            app.init_asset_loader::<MapAssetLoader>();
        }
    }
}
