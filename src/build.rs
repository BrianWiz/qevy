use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
use std::collections::BTreeMap;

use crate::components::*;
use crate::conversions::*;

use crate::{MapAsset, PostBuildMapEvent};

#[cfg(feature = "xpbd")]
use bevy_xpbd_3d::prelude::*;

#[cfg(feature = "rapier")]
use bevy_rapier3d::prelude::*;

pub fn build_map(
    map_entity: Entity,
    map_asset: &mut MapAsset,
    meshes: &mut Assets<Mesh>,
    commands: &mut Commands,
    ev_post_build_map: &mut EventWriter<PostBuildMapEvent>,
) {
    let geomap = map_asset.geomap.as_ref().unwrap();

    let face_trangle_planes = &geomap.face_planes;
    let face_planes = shambler::face::face_planes(&face_trangle_planes);
    let brush_hulls = shambler::brush::brush_hulls(&geomap.brush_faces, &face_planes);
    let (face_vertices, _face_vertex_planes) =
        shambler::face::face_vertices(&geomap.brush_faces, &face_planes, &brush_hulls);
    let face_centers = shambler::face::face_centers(&face_vertices);
    let face_indices = shambler::face::face_indices(
        &geomap.face_planes,
        &face_planes,
        &face_vertices,
        &face_centers,
        shambler::face::FaceWinding::Clockwise,
    );
    let face_triangle_indices = shambler::face::face_triangle_indices(&face_indices);
    let face_normals = shambler::face::normals_flat(&face_vertices, &face_planes);

    let face_uvs = shambler::face::new(
        &geomap.faces,
        &geomap.textures,
        &geomap.face_textures,
        &face_vertices,
        &face_planes,
        &geomap.face_offsets,
        &geomap.face_angles,
        &geomap.face_scales,
        &shambler::texture::texture_sizes(
            &geomap.textures,
            map_asset.get_texture_names_with_size(),
        ),
    );

    // spawn entities (@PointClass)
    geomap
        .entity_properties
        .iter()
        .for_each(|(entity_id, props)| {
            // if it's an entity brush we process it later
            if geomap.entity_brushes.get(entity_id).is_some() {
                return;
            }

            // map properties into btree
            // just easier to access props
            let mut props = props
                .iter()
                .map(|p| (p.key.as_str(), p.value.as_str()))
                .collect::<BTreeMap<_, _>>();

            let classname = props.get(&"classname").unwrap_or(&"").to_string();
            let translation = props.get(&"origin").unwrap_or(&"0 0 0").to_string();
            let rotation = props.get(&"angles").unwrap_or(&"0 0 0").to_string();

            let translation = translation.split(" ").collect::<Vec<&str>>();
            let translation = if translation.len() == 3 {
                to_bevy_position(&Vec3::new(
                    translation[0].parse::<f32>().unwrap(),
                    translation[1].parse::<f32>().unwrap(),
                    translation[2].parse::<f32>().unwrap(),
                ))
            } else {
                Vec3::ZERO
            };

            let rotation = rotation.split(" ").collect::<Vec<&str>>();
            let rotation = if rotation.len() == 3 {
                to_bevy_rotation(&Vec3::new(
                    rotation[0].parse::<f32>().unwrap(),
                    rotation[1].parse::<f32>().unwrap(),
                    rotation[2].parse::<f32>().unwrap(),
                ))
            } else {
                Quat::IDENTITY
            };

            commands.entity(map_entity).with_children(|children| {
                let mut entity = children.spawn((MapEntityProperties {
                    classname: classname.to_string(),
                    transform: Transform::from_translation(translation)
                        * Transform::from_rotation(rotation),
                    properties: props
                        .iter_mut()
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect(),
                },));

                if let Some(target_name) = props.get("targetname") {
                    entity.insert(TriggerTarget {
                        target_name: target_name.to_string(),
                    });
                }
            });
        });

    // spawn brush entities (@SolidClass)
    for (entity_id, brushes) in geomap.entity_brushes.iter() {
        let entity_properties = geomap.entity_properties.get(&entity_id);

        if let None = entity_properties {
            panic!("brush entity {} has no properties!", entity_id);
        }

        // map properties into btree
        // just easier to access props
        let mut props = entity_properties
            .unwrap()
            .iter()
            .map(|p| (p.key.as_str(), p.value.as_str()))
            .collect::<BTreeMap<_, _>>();
        let classname = props.get(&"classname").unwrap_or(&"").to_string();
        let brush_entity = (
            BrushEntity {},
            MapEntityProperties {
                classname: classname.to_string(),
                properties: props
                    .iter_mut()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                ..default()
            },
            SpatialBundle::default(),
        );

        commands.entity(map_entity).with_children(|children| {
            let mut entity = children.spawn(brush_entity);
            entity.with_children(|gchildren| {
                for brush_id in brushes.iter() {
                    let brush_faces = geomap.brush_faces.get(brush_id).unwrap();
                    let mut brush_vertices: Vec<Vec3> = Vec::new();

                    for face_id in brush_faces.iter() {
                        let texture_id = geomap.face_textures.get(face_id).unwrap();
                        let texture_name = geomap.textures.get(texture_id).unwrap();

                        let indices =
                            to_bevy_indecies(&face_triangle_indices.get(&face_id).unwrap());
                        let vertices = to_bevy_vertices(&face_vertices.get(&face_id).unwrap());
                        let normals = to_bevy_vec3s(&face_normals.get(&face_id).unwrap());
                        let uvs = uvs_to_bevy_vec2s(&face_uvs.get(&face_id).unwrap());
                        brush_vertices.extend(vertices.clone());

                        // we don't render anything for these two textures
                        if texture_name == "trigger"
                            || texture_name == "clip"
                            || texture_name == "common/trigger"
                            || texture_name == "common/clip"
                        {
                            continue;
                        }

                        let mut mesh = Mesh::new(
                            PrimitiveTopology::TriangleList,
                            RenderAssetUsages::RENDER_WORLD,
                        );
                        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
                        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
                        mesh.insert_indices(Indices::U32(indices));

                        if uvs.len() > 0 {
                            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
                            if let Err(e) = mesh.generate_tangents() {
                                println!("error generating tangents: {:?}", e);
                            }
                        }

                        if map_asset.material_handles.contains_key(texture_name) {
                            gchildren.spawn(PbrBundle {
                                mesh: meshes.add(mesh),
                                material: map_asset
                                    .material_handles
                                    .get(texture_name)
                                    .unwrap()
                                    .clone(),
                                ..default()
                            });
                        }
                    }

                    // spawn it's collider
                    #[cfg(feature = "xpbd")]
                    {
                        if let Some(convex_hull) =
                            bevy_xpbd_3d::prelude::Collider::convex_hull(brush_vertices)
                        {
                            let mut collider =
                                gchildren.spawn((convex_hull, TransformBundle::default()));
                            if classname == "trigger_multiple" {
                                collider.insert((
                                    TriggerMultiple {
                                        target: props.get("target").unwrap().to_string(),
                                    },
                                    bevy_xpbd_3d::prelude::RigidBody::Dynamic,
                                    bevy_xpbd_3d::prelude::Sensor,
                                ));
                            } else if classname == "trigger_once" {
                                collider.insert((
                                    TriggerOnce {
                                        target: props.get("target").unwrap().to_string(),
                                    },
                                    bevy_xpbd_3d::prelude::RigidBody::Dynamic,
                                    bevy_xpbd_3d::prelude::Sensor,
                                ));
                            } else {
                                collider.insert((bevy_xpbd_3d::prelude::RigidBody::Static,));
                            }
                        }
                    }

                    #[cfg(feature = "rapier")]
                    {
                        if let Some(convex_hull) =
                            bevy_rapier3d::prelude::Collider::convex_hull(&brush_vertices)
                        {
                            let mut collider =
                                gchildren.spawn((convex_hull, TransformBundle::default()));
                            if classname == "trigger_multiple" {
                                collider.insert((
                                    TriggerMultiple {
                                        target: props.get("target").unwrap().to_string(),
                                    },
                                    bevy_rapier3d::prelude::RigidBody::Dynamic,
                                    bevy_rapier3d::prelude::Sensor,
                                ));
                            } else if classname == "trigger_once" {
                                collider.insert((
                                    TriggerOnce {
                                        target: props.get("target").unwrap().to_string(),
                                    },
                                    bevy_rapier3d::prelude::RigidBody::Dynamic,
                                    bevy_rapier3d::prelude::Sensor,
                                ));
                            } else {
                                collider.insert((bevy_rapier3d::prelude::RigidBody::Fixed,));
                            }
                        }
                    }
                }
            });

            if let Some(target_name) = props.get("targetname") {
                entity.insert(TriggerTarget {
                    target_name: target_name.to_string(),
                });
            }
        });
    }

    ev_post_build_map.send(PostBuildMapEvent { map: map_entity });
}

// pub fn cleanup_spawned_entities_system(
//     mut commands: Commands,
//     q_spawning_entities: Query<Entity, With<MapEntityProperties>>,
// ) {
//     for entity in q_spawning_entities.iter() {
//         commands.entity(entity).remove::<MapEntityProperties>();
//     }
// }
