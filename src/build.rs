use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use std::collections::BTreeMap;

use crate::components::MapEntityProperties;
use crate::conversions::{
    to_bevy_indecies, to_bevy_position, to_bevy_rotation, to_bevy_vec3s, to_bevy_vertices,
    uvs_to_bevy_vec2s,
};
use crate::{MapAsset, PostMapBuildHook};

#[cfg(feature = "xpbd")]
use bevy_xpbd_3d::prelude::*;

pub fn build_map(
    map_entity: Entity,
    map_asset: &mut MapAsset,
    meshes: &mut Assets<Mesh>,
    commands: &mut Commands,
    post_build_hook: &mut ResMut<PostMapBuildHook>,
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

    // spawn faces
    for face_id in geomap.faces.iter() {
        let texture_id = geomap.face_textures.get(face_id).unwrap();
        let texture_name = geomap.textures.get(texture_id).unwrap();
        // we only care about faces with textures
        if !map_asset.material_handles.contains_key(texture_name) {
            continue;
        }

        let indices = to_bevy_indecies(&face_triangle_indices.get(&face_id).unwrap());
        let vertices = to_bevy_vertices(&face_vertices.get(&face_id).unwrap());
        let normals = to_bevy_vec3s(&face_normals.get(&face_id).unwrap());
        let uvs = uvs_to_bevy_vec2s(&face_uvs.get(&face_id).unwrap());

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_indices(Some(Indices::U32(indices)));

        if uvs.len() > 0 {
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
            if let Err(e) = mesh.generate_tangents() {
                println!("error generating tangents: {:?}", e);
            }
        }

        commands.entity(map_entity).with_children(|children| {
            children.spawn(PbrBundle {
                mesh: meshes.add(mesh),
                material: map_asset
                    .material_handles
                    .get(texture_name)
                    .unwrap()
                    .clone(),
                ..default()
            });
        });
    }

    // spawn colliders
    for brush_id in geomap.brushes.iter() {
        let brush_faces = geomap.brush_faces.get(brush_id).unwrap();
        let mut brush_vertices: Vec<Vec3> = Vec::new();
        for face_id in brush_faces.iter() {
            let vertices = to_bevy_vertices(&face_vertices.get(&face_id).unwrap());
            brush_vertices.extend(vertices);
        }
        #[cfg(feature = "xpbd")]
        {
            let convex_hull = Collider::convex_hull(brush_vertices);
            if convex_hull.is_some() {
                commands.spawn((RigidBody::Static, convex_hull.unwrap()));
            }
        }
    }

    // spawn entities
    geomap.entity_properties.iter().for_each(|(_, props)| {
        // map properties into btree
        // just easier to access props
        let mut props = props
            .iter()
            .map(|p| (p.key.as_str(), p.value.as_str()))
            .collect::<BTreeMap<_, _>>();

        commands.spawn((MapEntityProperties { ..default() },));

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

        props.remove_entry(&"classname");
        props.remove_entry(&"origin");
        props.remove_entry(&"angles");

        commands.entity(map_entity).with_children(|children| {
            children.spawn((MapEntityProperties {
                classname: classname.to_string(),
                transform: Transform::from_translation(translation)
                    * Transform::from_rotation(rotation),
                properties: props
                    .iter_mut()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            },));
        });
    });

    if let Some(system) = post_build_hook.system {
        commands.run_system(system);
    }
}

pub fn cleanup_spawned_entities_system(
    mut commands: Commands,
    q_spawning_entities: Query<Entity, With<MapEntityProperties>>,
) {
    for entity in q_spawning_entities.iter() {
        commands.entity(entity).remove::<MapEntityProperties>();
    }
}
