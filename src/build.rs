use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
#[cfg(feature = "rapier")]
use bevy_rapier3d::geometry::ActiveCollisionTypes;
use std::collections::BTreeMap;
use std::time::Duration;

use crate::components::*;
use crate::conversions::*;

use crate::{MapAsset, PostBuildMapEvent};

#[derive(Event)]
pub struct SpawnMeshEvent {
    map: Entity,
    mesh: Mesh,
    collider: Option<Entity>,
    material: Handle<StandardMaterial>,
}

pub fn build_map(
    map_units: &MapUnits,
    map_entity: Entity,
    map_asset: &mut MapAsset,
    commands: &mut Commands,
    spawn_mesh_event: &mut EventWriter<SpawnMeshEvent>,
    post_build_map_event: &mut EventWriter<PostBuildMapEvent>,
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
                to_bevy_position(
                    &Vec3::new(
                        translation[0].parse::<f32>().unwrap(),
                        translation[1].parse::<f32>().unwrap(),
                        translation[2].parse::<f32>().unwrap(),
                    ),
                    &map_units,
                )
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

                    let mut meshes_to_spawn = Vec::new();

                    for face_id in brush_faces.iter() {
                        let texture_id = geomap.face_textures.get(face_id).unwrap();
                        let texture_name = geomap.textures.get(texture_id).unwrap();

                        let indices =
                            to_bevy_indecies(&face_triangle_indices.get(&face_id).unwrap());
                        let vertices =
                            to_bevy_vertices(&face_vertices.get(&face_id).unwrap(), &map_units);
                        let normals = to_bevy_vec3s(&face_normals.get(&face_id).unwrap());
                        let uvs = uvs_to_bevy_vec2s(&face_uvs.get(&face_id).unwrap());
                        brush_vertices.extend(vertices.clone());

                        // we don't render anything for these textures
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

                        meshes_to_spawn.push((mesh, texture_name));
                    }

                    // spawn it's collider
                    #[cfg(feature = "avian")]
                    {
                        if let Some(convex_hull) =
                            avian3d::prelude::Collider::convex_hull(brush_vertices)
                        {
                            let mut collider = gchildren.spawn((
                                convex_hull,
                                TransformBundle::default(),
                                VisibilityBundle::default(),
                            ));
                            if classname == "trigger_multiple" {
                                collider.insert((
                                    TriggerMultiple {
                                        target: props.get("target").unwrap().to_string(),
                                    },
                                    avian3d::prelude::RigidBody::Dynamic,
                                    avian3d::prelude::Sensor,
                                ));
                            } else if classname == "trigger_once" {
                                collider.insert((
                                    TriggerOnce {
                                        target: props.get("target").unwrap().to_string(),
                                    },
                                    avian3d::prelude::RigidBody::Dynamic,
                                    avian3d::prelude::Sensor,
                                ));
                            } else {
                                collider.insert((avian3d::prelude::RigidBody::Static,));
                            }

                            for (mesh, texture_name) in meshes_to_spawn {
                                if map_asset.material_handles.contains_key(texture_name) {
                                    spawn_mesh_event.send(SpawnMeshEvent {
                                        map: map_entity,
                                        mesh: mesh,
                                        collider: Some(collider.id()),
                                        material: map_asset
                                            .material_handles
                                            .get(texture_name)
                                            .unwrap()
                                            .clone(),
                                    });
                                }
                            }
                        }
                    }

                    #[cfg(feature = "rapier")]
                    #[cfg(not(feature = "avian"))]
                    {
                        if let Some(convex_hull) =
                            bevy_rapier3d::prelude::Collider::convex_hull(&brush_vertices)
                        {
                            let mut collider = gchildren.spawn((
                                convex_hull,
                                TransformBundle::default(),
                                VisibilityBundle::default(),
                            ));
                            if classname == "trigger_multiple" {
                                collider.insert((
                                    TriggerMultiple {
                                        target: props.get("target").unwrap().to_string(),
                                    },
                                    bevy_rapier3d::prelude::RigidBody::KinematicPositionBased,
                                    bevy_rapier3d::prelude::Sensor,
                                    ActiveCollisionTypes::default()
                                        | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
                                ));
                            } else if classname == "trigger_once" {
                                collider.insert((
                                    TriggerOnce {
                                        target: props.get("target").unwrap().to_string(),
                                    },
                                    bevy_rapier3d::prelude::RigidBody::KinematicPositionBased,
                                    bevy_rapier3d::prelude::Sensor,
                                    ActiveCollisionTypes::default()
                                        | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
                                ));
                            } else {
                                collider.insert((bevy_rapier3d::prelude::RigidBody::Fixed,));
                            }

                            for (mesh, texture_name) in meshes_to_spawn {
                                if map_asset.material_handles.contains_key(texture_name) {
                                    spawn_mesh_event.send(SpawnMeshEvent {
                                        map: map_entity,
                                        mesh: mesh,
                                        collider: Some(collider.id()),
                                        material: map_asset
                                            .material_handles
                                            .get(texture_name)
                                            .unwrap()
                                            .clone(),
                                    });
                                }
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

    post_build_map_event.send(PostBuildMapEvent { map: map_entity });
}

pub fn mesh_spawn_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut spawn_mesh_event: EventReader<SpawnMeshEvent>,
) {
    for ev in spawn_mesh_event.read() {
        // if this mesh has a collider, make it a child of the collider
        if let Some(collider) = ev.collider {
            commands.entity(collider).with_children(|children| {
                children.spawn(PbrBundle {
                    mesh: meshes.add(ev.mesh.to_owned()),
                    material: ev.material.to_owned(),
                    ..default()
                });
            });
        // otherwise, it's a child of the map
        } else {
            commands.entity(ev.map).with_children(|children| {
                children.spawn(PbrBundle {
                    mesh: meshes.add(ev.mesh.to_owned()),
                    material: ev.material.to_owned(),
                    ..default()
                });
            });
        }
    }
}

pub fn post_build_map_system(
    map_units: Res<MapUnits>,
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
                    let mut mover_entity = commands.entity(entity);
                    mover_entity.insert((
                        Mover {
                            moving_time: Duration::from_secs_f32(
                                props.get_property_as_f32("moving_time", 1.0),
                            ),
                            destination_time: Duration::from_secs_f32(
                                props.get_property_as_f32("destination_time", 2.0),
                            ),
                            destination_offset: {
                                to_bevy_position(
                                    &props.get_property_as_vec3("destination_offset", Vec3::ZERO),
                                    &map_units,
                                )
                            },
                            state: MoverState::default(),
                        },
                        TransformBundle {
                            local: Transform::from_xyz(0.0, 0.0, 0.0),
                            ..default()
                        },
                    ));

                    if let Some(mover_kind) =
                        props.get_property_as_string("mover_kind", Some(&"linear".into()))
                    {
                        match mover_kind.as_str() {
                            "door" => {
                                mover_entity.insert(Door {
                                    key: props.get_property_as_string("key", None).into(),
                                    open_once: props.get_property_as_bool("open_once", false),
                                });
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
