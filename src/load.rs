use crate::build::SpawnMeshEvent;
use crate::components::*;
use crate::{MapAsset, PostBuildMapEvent};
use bevy::asset::LoadContext;
use bevy::asset::LoadedAsset;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::texture::CompressedImageFormats;
use bevy::render::texture::ImageAddressMode;
use bevy::render::texture::ImageSampler;
use bevy::render::texture::ImageSamplerDescriptor;
use bevy::render::texture::ImageType;

pub(crate) fn handle_loaded_map_system(
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

pub(crate) async fn load_map<'a>(map_asset: &mut MapAsset, load_context: &mut LoadContext<'a>) {
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
                    perceptual_roughness: 0.65,
                    metallic: 0.8,
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

pub fn post_build_map_system(
    mut commands: Commands,
    mut event_reader: EventReader<crate::PostBuildMapEvent>,
    mut map_entities: Query<(Entity, &crate::components::MapEntityProperties)>,
) {
    for _ in event_reader.read() {
        // to set these up, see the .fgd file in the TrenchBroom
        // game folder for Qevy Example also see the readme
        for (entity, props) in map_entities.iter_mut() {
            match props.classname.as_str() {
                "light" => {
                    commands.entity(entity).insert(PointLightBundle {
                        transform: props.transform,
                        point_light: PointLight {
                            color: props.get_property_as_color("color", Color::WHITE),
                            radius: props.get_property_as_f32("radius", 0.0),
                            range: props.get_property_as_f32("range", 10.0),
                            intensity: props.get_property_as_f32("intensity", 800.0),
                            shadows_enabled: props.get_property_as_bool("shadows_enabled", false),
                            ..default()
                        },
                        ..default()
                    });
                }
                "directional_light" => {
                    commands.entity(entity).insert(DirectionalLightBundle {
                        transform: props.transform,
                        directional_light: DirectionalLight {
                            color: props.get_property_as_color("color", Color::WHITE),
                            illuminance: props.get_property_as_f32("illuminance", 10000.0),
                            shadows_enabled: props.get_property_as_bool("shadows_enabled", false),
                            ..default()
                        },
                        ..default()
                    });
                }
                "mover" => {
                    let mover_type = props.get_property_as_string("mover_type", &"linear".into());
                    let mut mover_entity = commands.entity(entity);
                    mover_entity.insert((
                        Mover {
                            speed: props.get_property_as_f32("speed", 1.0),
                            destination_translation: props
                                .get_property_as_vec3("translation", Vec3::ZERO),
                            start_translation: Vec3::ZERO,
                        },
                        TransformBundle {
                            local: Transform::from_xyz(0.0, 0.0, 0.0),
                            ..default()
                        },
                    ));

                    if mover_type == "door" {
                        let open_once = props.get_property_as_bool("open_once", false);
                        let open_time = props.get_property_as_i32("open_time", 1000);
                        mover_entity.insert(Door {
                            open_time: std::time::Duration::from_millis(open_time as u64),
                            triggered_time: None,
                            key: props.get_property_as_string("key", &"".into()).into(),
                            open_once: open_once,
                        });
                    }
                }
                _ => {}
            }
        }
    }
}
