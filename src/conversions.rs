use bevy::prelude::*;
use shambler::{Vector2, Vector3};

use crate::components::MapUnits;

pub const SHAMBLER_UNITS_TO_BEVY_METERS: f32 = 0.03125; // 1 meter = 32 units (1/32)

pub fn to_bevy_indecies(indecies: &Vec<usize>) -> Vec<u32> {
    let mut bevy_indecies: Vec<u32> = Vec::new();
    for index in indecies {
        bevy_indecies.push(*index as u32);
    }
    bevy_indecies
}

pub fn to_bevy_position(vector: &Vec3, map_units: &MapUnits) -> Vec3 {
    match map_units {
        MapUnits::Bevy => Vec3::new(vector.y, vector.z, vector.x) * SHAMBLER_UNITS_TO_BEVY_METERS,
        MapUnits::Trenchbroom => Vec3::new(vector.y, vector.z, vector.x),
    }
}

pub fn to_bevy_rotation(rotation: &Vec3) -> Quat {
    Quat::from_euler(
        bevy::math::EulerRot::YXZ,
        rotation.y.to_radians(),
        rotation.x.to_radians(),
        rotation.z.to_radians(),
    ) // * Quat::from_axis_angle(Vec3::Y, -90.0_f32.to_radians())
}

pub fn to_bevy_vertices(vertices: &Vec<Vector3>, map_units: &MapUnits) -> Vec<Vec3> {
    let mut bevy_vertices: Vec<Vec3> = Vec::new();
    for vertex in vertices {
        match map_units {
            MapUnits::Bevy => bevy_vertices
                .push(Vec3::new(vertex.y, vertex.z, vertex.x) * SHAMBLER_UNITS_TO_BEVY_METERS),
            MapUnits::Trenchbroom => bevy_vertices.push(Vec3::new(vertex.y, vertex.z, vertex.x)),
        }
    }
    bevy_vertices
}

pub fn to_bevy_vec3s(normals: &Vec<Vector3>) -> Vec<Vec3> {
    let mut bevy_normals: Vec<Vec3> = Vec::new();
    for normal in normals {
        bevy_normals.push(Vec3::new(normal.y, normal.z, normal.x));
    }
    bevy_normals
}

pub fn uvs_to_bevy_vec2s(uvs: &Vec<Vector2>) -> Vec<Vec2> {
    let mut bevy_uvs: Vec<Vec2> = Vec::new();
    for uv in uvs {
        bevy_uvs.push(Vec2::new(uv.x, uv.y));
    }
    bevy_uvs
}
