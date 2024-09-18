use crate::build::SpawnMeshEvent;
use crate::{components::*, MapAssetLoaderError};
use crate::{MapAsset, PostBuildMapEvent};
use bevy::asset::io::Reader;
use bevy::asset::AsyncReadExt;
use bevy::asset::LoadContext;
use bevy::asset::LoadedAsset;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::texture::CompressedImageFormats;
use bevy::render::texture::ImageAddressMode;
use bevy::render::texture::ImageSampler;
use bevy::render::texture::ImageSamplerDescriptor;
use bevy::render::texture::ImageType;
use std::collections::BTreeMap;

pub(crate) fn extensions() -> &'static [&'static str] {
    &["map"]
}

pub(crate) async fn load<'a>(
    reader: &'a mut Reader<'_>,
    load_context: &'a mut LoadContext<'_>,
    headless: bool,
) -> Result<MapAsset, MapAssetLoaderError> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).await?;
    if let Ok(map) = std::str::from_utf8(&bytes)
        .expect("invalid utf8")
        .parse::<shalrath::repr::Map>()
    {
        let geomap = Some(shambler::GeoMap::new(map.clone()));
        let mut map = MapAsset {
            geomap: geomap,
            texture_sizes: BTreeMap::new(),
            material_handles: BTreeMap::new(),
        };

        if !headless {
            load_map_textures(&mut map, load_context).await;
        }
        return Ok(map);
    }
    Err(MapAssetLoaderError::Io(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "invalid map",
    )))
}

pub(crate) fn handle_loaded_map_system(
    map_units: Res<MapUnits>,
    mut commands: Commands,
    mut map_assets: ResMut<Assets<MapAsset>>,
    mut ev_asset: EventReader<AssetEvent<MapAsset>>,
    mut q_maps: Query<Entity, With<Map>>,
    mut post_build_event: EventWriter<PostBuildMapEvent>,
    mut spawn_mesh_event: EventWriter<SpawnMeshEvent>,
) {
    for ev in ev_asset.read() {
        match ev {
            AssetEvent::LoadedWithDependencies { id } => {
                for map_entity in q_maps.iter_mut() {
                    commands.entity(map_entity).despawn_descendants();
                    let map_asset = map_assets.get_mut(*id).unwrap();
                    crate::build::build_map(
                        &map_units,
                        map_entity,
                        map_asset,
                        &mut commands,
                        &mut spawn_mesh_event,
                        &mut post_build_event,
                    );
                }
            }
            _ => {}
        }
    }
}

pub(crate) async fn load_map_textures<'a>(
    map_asset: &mut MapAsset,
    load_context: &mut LoadContext<'a>,
) {
    let geomap = map_asset.geomap.as_mut().unwrap();

    // for each texture, load it into the asset server
    for texture_info in geomap.textures.iter() {
        let texture_name = texture_info.1;
        let file = format!("textures/{}.png", texture_name);

        let bytes = load_context.read_asset_bytes(&file).await;

        if let Ok(bytes) = bytes {
            let texture = Image::from_buffer(
                &bytes,
                ImageType::Extension("png"),
                CompressedImageFormats::all(),
                false,
                ImageSampler::Descriptor(ImageSamplerDescriptor {
                    address_mode_u: ImageAddressMode::Repeat,
                    address_mode_v: ImageAddressMode::Repeat,
                    ..default()
                }),
                RenderAssetUsages::RENDER_WORLD,
            );

            if texture.is_ok() {
                let texture = texture.unwrap();
                let texture_handle = load_context.add_loaded_labeled_asset(
                    format!("textures/{}", texture_name),
                    LoadedAsset::from(texture.clone()),
                );
                let mat = StandardMaterial {
                    base_color_texture: Some(texture_handle),
                    perceptual_roughness: 0.55,
                    metallic: 0.5,
                    ..default()
                };
                let mat_handle = load_context.add_loaded_labeled_asset::<StandardMaterial>(
                    format!("materials/{}", texture_name),
                    LoadedAsset::from(mat),
                );
                map_asset
                    .material_handles
                    .insert(texture_name.clone(), mat_handle);
                map_asset
                    .texture_sizes
                    .insert(texture_name.clone(), (texture.width(), texture.height()));
            }
        }
    }
}
