use avian3d::prelude::*;
use bevy::audio::Decodable;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::{input::mouse::MouseMotion, window::CursorGrabMode};

use qevy::{components::*, PostBuildMapEvent};

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
            qevy::MapAssetLoaderPlugin::default(),
            PhysicsPlugins::default(), // Avian
            PhysicsDebugPlugin::default(),
        ))
        .add_systems(Startup, (spawn_map, spawn_character))
        .add_systems(
            Update,
            (
                movement,
                grab_mouse,
                my_post_build_map_system,
                door_system,
                qevy::load::post_build_map_system,
                qevy::gameplay_systems::avian_trigger_system,
            ),
        )
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
    mut map_entities: Query<(Entity, &MapEntityProperties)>,
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
        TriggerInstigator,
        RigidBody::Dynamic,
        GravityScale(0.0),
        Rotation(Quat::IDENTITY),
        Collider::sphere(0.5),
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
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut cameras: Query<&mut Transform, (With<Camera3d>, Without<Character>)>,
    mut characters: Query<(&Transform, &mut LinearVelocity, &mut Rotation), With<Character>>,
) {
    for (collider_transform, mut linear_velocity, mut rotation) in &mut characters {
        for mut camera_transform in &mut cameras {
            // Directional movement
            if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
                linear_velocity.0 -= rotation.0 * MOVE_SPEED * Vec3::Z;
            }
            if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
                linear_velocity.0 += rotation.0 * MOVE_SPEED * Vec3::Z;
            }
            if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
                linear_velocity.0 -= rotation.0 * MOVE_SPEED * Vec3::X;
            }
            if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight)
            {
                linear_velocity.0 += rotation.0 * MOVE_SPEED * Vec3::X;
            }
            if keyboard_input.pressed(KeyCode::KeyQ) || keyboard_input.pressed(KeyCode::Space) {
                linear_velocity.y += MOVE_SPEED;
            }
            if keyboard_input.pressed(KeyCode::KeyX) || keyboard_input.pressed(KeyCode::ShiftLeft) {
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

pub fn door_system(
    time: Res<Time>,
    mut ev_reader: EventReader<TriggeredEvent>,
    mut q_doors: Query<(&mut Door, &TriggerTarget, &mut Transform, &Mover)>,
) {
    for triggered_event in ev_reader.read() {
        for (mut door, trigger_target, _, _) in q_doors.iter_mut() {
            if trigger_target.target_name == triggered_event.target {
                door.triggered_time = Some(std::time::Instant::now());
            }
        }
    }

    for (mut door, _, mut transform, mover) in q_doors.iter_mut() {
        let triggered = if door.open_once {
            door.triggered_time.is_some()
        } else {
            door.triggered_time.is_some()
                && (std::time::Instant::now() - door.triggered_time.unwrap() < door.open_time)
        };

        // open
        if triggered {
            let destination = mover.destination_translation;
            let direction = destination - transform.translation;
            let move_distance = mover.speed * time.delta_seconds();

            if direction.length() < move_distance {
                transform.translation = destination;
            } else {
                transform.translation += direction.normalize() * move_distance;
            }
        // close
        } else {
            if !door.open_once && door.triggered_time.is_some() {
                continue;
            }
            door.triggered_time = None;

            let destination = mover.start_translation;
            let direction = destination - transform.translation;
            let move_distance = mover.speed * time.delta_seconds();

            if direction.length() < move_distance {
                transform.translation = destination;
            } else {
                transform.translation += direction.normalize() * move_distance;
            }
        }
    }
}

fn grab_mouse(
    mut windows: Query<&mut Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
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
