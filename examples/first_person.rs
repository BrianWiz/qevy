use std::time::Duration;

#[cfg(feature = "avian")]
use avian3d::prelude::*;

#[cfg(feature = "rapier")]
#[cfg(not(feature = "avian"))]
use bevy_rapier3d::prelude::*;

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
            // Avian
            #[cfg(feature = "avian")]
            PhysicsPlugins::default(),
            // PhysicsDebugPlugin::default(),
            // Rapier
            #[cfg(feature = "rapier")]
            #[cfg(not(feature = "avian"))]
            RapierPhysicsPlugin::<NoUserData>::default(),
            // RapierDebugRenderPlugin::default()
        ))
        .add_systems(Startup, (spawn_map, spawn_character))
        .add_systems(
            Update,
            (
                movement,
                grab_mouse,
                my_post_build_map_system,
                door_system,
                qevy::build::post_build_map_system,
                // Avian
                #[cfg(feature = "avian")]
                qevy::gameplay_systems::avian_trigger_system,
                // Rapier
                #[cfg(feature = "rapier")]
                #[cfg(not(feature = "avian"))]
                qevy::gameplay_systems::rapier_trigger_system,
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
                    commands.entity(entity).insert((
                        props.transform,
                        Mesh3d(asset_server.load("models/monkey.gltf#Mesh0/Primitive0")),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgb(0.5, 0.5, 0.5),
                            ..default()
                        })),
                    ));
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
    let mut character = commands.spawn((
        Character,
        TriggerInstigator,
        RigidBody::Dynamic,
        GravityScale(0.0),
        Rotation(Quat::IDENTITY),
        TransformBundle {
            local: Transform::from_xyz(0.0, 5.0, 0.0),
            ..default()
        },
    ));

    #[cfg(feature = "avian")]
    character.insert((Collider::sphere(0.5),));

    #[cfg(feature = "rapier")]
    #[cfg(not(feature = "avian"))]
    character.insert((Collider::ball(0.5), Velocity::default()));

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
    #[cfg(feature = "avian")] mut characters: Query<
        (&Transform, &mut LinearVelocity, &mut Rotation),
        With<Character>,
    >,

    #[cfg(feature = "rapier")]
    #[cfg(not(feature = "avian"))]
    mut characters: Query<(&Transform, &mut Velocity, &mut Rotation), With<Character>>,
) {
    for (collider_transform, mut velocity, mut rotation) in &mut characters {
        #[cfg(feature = "avian")]
        let linear_velocity = &mut velocity.0;

        #[cfg(feature = "rapier")]
        #[cfg(not(feature = "avian"))]
        let linear_velocity = &mut velocity.linvel;

        for mut camera_transform in &mut cameras {
            // Directional movement
            if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
                *linear_velocity -= rotation.0 * MOVE_SPEED * Vec3::Z;
            }
            if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
                *linear_velocity += rotation.0 * MOVE_SPEED * Vec3::Z;
            }
            if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
                *linear_velocity -= rotation.0 * MOVE_SPEED * Vec3::X;
            }
            if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight)
            {
                *linear_velocity += rotation.0 * MOVE_SPEED * Vec3::X;
            }
            if keyboard_input.pressed(KeyCode::KeyQ) || keyboard_input.pressed(KeyCode::Space) {
                linear_velocity.y += MOVE_SPEED;
            }
            if keyboard_input.pressed(KeyCode::KeyX) || keyboard_input.pressed(KeyCode::ShiftLeft) {
                linear_velocity.y -= MOVE_SPEED;
            }

            // Slow player down
            *linear_velocity *= 0.82;

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
                (1.0 / 60.0 * 1000.0) * time.delta_secs(),
            );
        }
    }
}

pub fn door_system(
    time: Res<Time>,
    mut ev_reader: EventReader<TriggeredEvent>,
    mut q_doors: Query<(&mut Door, &TriggerTarget, &mut Transform, &mut Mover)>,
) {
    for triggered_event in ev_reader.read() {
        for (_, trigger_target, _, mut mover) in q_doors.iter_mut() {
            if trigger_target.target_name == triggered_event.target {
                match mover.state {
                    MoverState::AtStart => {
                        mover.state = MoverState::MovingToDestination(Timer::new(
                            mover.moving_time,
                            TimerMode::Once,
                        ))
                    }
                    MoverState::AtDestination(_) => {
                        mover.state = MoverState::AtDestination(Timer::new(
                            mover.destination_time,
                            TimerMode::Once,
                        ))
                    }
                    _ => {}
                }
            }
        }
    }

    let delta = time.delta();
    for (door, _, mut transform, mut mover) in q_doors.iter_mut() {
        let destination_offset = mover.destination_offset;
        let moving_time_secs = mover.moving_time.as_secs_f32();
        match &mut mover.state {
            MoverState::AtStart => {}
            MoverState::MovingToDestination(ref mut timer) => {
                timer.tick(delta);
                transform.translation +=
                    destination_offset / moving_time_secs * delta.as_secs_f32();
                if timer.just_finished() {
                    if door.open_once {
                        mover.state =
                            MoverState::AtDestination(Timer::new(Duration::MAX, TimerMode::Once))
                    } else {
                        mover.state = MoverState::AtDestination(Timer::new(
                            mover.destination_time,
                            TimerMode::Once,
                        ))
                    }
                }
            }
            MoverState::AtDestination(ref mut timer) => {
                timer.tick(delta);
                if timer.just_finished() {
                    mover.state =
                        MoverState::MovingToStart(Timer::new(mover.moving_time, TimerMode::Once))
                }
            }
            MoverState::MovingToStart(ref mut timer) => {
                timer.tick(delta);
                transform.translation -=
                    destination_offset / moving_time_secs * delta.as_secs_f32();
                if timer.just_finished() {
                    mover.state = MoverState::AtStart;
                }
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
        window.cursor_options.visible = false;
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
    }

    if key.just_pressed(KeyCode::Escape) {
        window.cursor_options.visible = true;
        window.cursor_options.grab_mode = CursorGrabMode::None;
    }
}
