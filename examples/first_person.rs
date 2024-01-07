use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::{input::mouse::MouseMotion, window::CursorGrabMode};
use bevy_xpbd_3d::prelude::*;

use qevy::{components::MapEntityProperties, PostBuildMapEvent};

const MOVE_SPEED: f32 = 2.0;
const MOUSE_SENSITIVITY: f32 = 0.1;

#[derive(Component)]
struct Character;

#[derive(Component)]
struct Rotation(Quat);

#[derive(Component)]
pub struct SpawnPoint;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            qevy::MapAssetLoaderPlugin,
            PhysicsPlugins::default(), // XPBD
                                       //PhysicsDebugPlugin::default(),
        ))
        .add_systems(Startup, (spawn_map, spawn_character))
        .add_systems(Update, (movement, grab_mouse, my_post_build_map_system))
        .run();
}

fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    // spawn the map
    commands.spawn(qevy::components::MapBundle {
        map: qevy::components::Map {
            asset: asset_server.load("example.map"), // map must be under `assets` folder
            ..default()
        },
        ..default()
    });
}

pub fn my_post_build_map_system(
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    mut event_reader: EventReader<PostBuildMapEvent>,
    mut map_entities: Query<(Entity, &qevy::components::MapEntityProperties)>,
) {
    for _ in event_reader.read() {
        // to set these up, see the .fgd file in the TrenchBroom
        // game folder for Qevy Example also see the readme
        for (entity, props) in map_entities.iter_mut() {
            match props.classname.as_str() {
                "spawn_point" => {
                    commands.entity(entity).insert(TransformBundle {
                        local: props.transform,
                        ..default()
                    });
                }
                "light" => {
                    commands.entity(entity).insert(PointLightBundle {
                        transform: props.transform,
                        point_light: PointLight {
                            color: props.get_property_as_color("color", Color::WHITE),
                            radius: props.get_property_as_f32("radius", 0.0),
                            range: props.get_property_as_f32("range", 30.0),
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
                "monkey" => {
                    commands.entity(entity).insert(PbrBundle {
                        transform: props.transform,
                        mesh: asset_server.load("models/monkey.gltf#Mesh0/Primitive0"),
                        material: materials.add(StandardMaterial {
                            base_color: Color::rgb(0.5, 0.5, 0.5),
                            ..default()
                        }),
                        ..default()
                    });
                }
                _ => {}
            }
        }
    }
}

//==============================================================================
// all code below this is not related to Qevy
//==============================================================================

fn spawn_character(mut commands: Commands, mut q_windows: Query<&mut Window, With<PrimaryWindow>>) {
    // spawn the camera
    commands.spawn(Camera3dBundle {
        camera: Camera {
            hdr: true,
            ..default()
        },
        projection: Projection::Perspective(PerspectiveProjection {
            fov: 1.5708,
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // spawn the character
    commands.spawn((
        Character,
        RigidBody::Dynamic,
        GravityScale(0.0),
        Rotation(Quat::IDENTITY),
        Collider::ball(0.5),
        TransformBundle {
            local: Transform::from_xyz(0.0, 5.0, 0.0),
            ..default()
        },
    ));

    // center the cursor
    let mut window = q_windows.single_mut();
    let width = window.physical_width() / 2;
    let height = window.physical_height() / 2;
    window.set_physical_cursor_position(Some(UVec2::new(width, height).into()));
}

fn movement(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut cameras: Query<&mut Transform, (With<Camera3d>, Without<Character>)>,
    mut characters: Query<(&Transform, &mut LinearVelocity, &mut Rotation), With<Character>>,
) {
    for (collider_transform, mut linear_velocity, mut rotation) in &mut characters {
        for mut camera_transform in &mut cameras {
            // Directional movement
            if keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up) {
                linear_velocity.0 -= rotation.0 * MOVE_SPEED * Vec3::Z;
            }
            if keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down) {
                linear_velocity.0 += rotation.0 * MOVE_SPEED * Vec3::Z;
            }
            if keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left) {
                linear_velocity.0 -= rotation.0 * MOVE_SPEED * Vec3::X;
            }
            if keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right) {
                linear_velocity.0 += rotation.0 * MOVE_SPEED * Vec3::X;
            }
            if keyboard_input.pressed(KeyCode::Q) || keyboard_input.pressed(KeyCode::Space) {
                linear_velocity.y += MOVE_SPEED;
            }
            if keyboard_input.pressed(KeyCode::X) || keyboard_input.pressed(KeyCode::ShiftLeft) {
                linear_velocity.y -= MOVE_SPEED;
            }

            // Slow player down
            linear_velocity.0 *= 0.82;

            // FPS look
            let (mut yaw, mut pitch, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);
            for motion in mouse_motion.read() {
                pitch -= (MOUSE_SENSITIVITY * motion.delta.y).to_radians();
                yaw -= (MOUSE_SENSITIVITY * motion.delta.x).to_radians();
            }
            pitch = pitch.clamp(-1.54, 1.54);
            rotation.0 = Quat::from_axis_angle(Vec3::Y, yaw);
            camera_transform.rotation =
                Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);

            // lerp camera to collider because otherwise it jitters due to physics steps
            camera_transform.translation = camera_transform.translation.lerp(
                collider_transform.translation,
                (1.0 / 60.0 * 1000.0) * time.delta_seconds(),
            );
        }
    }
}

fn grab_mouse(
    mut windows: Query<&mut Window>,
    mouse: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
) {
    let mut window = windows.single_mut();

    if mouse.just_pressed(MouseButton::Left) {
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Locked;
    }

    if key.just_pressed(KeyCode::Escape) {
        window.cursor.visible = true;
        window.cursor.grab_mode = CursorGrabMode::None;
    }
}
