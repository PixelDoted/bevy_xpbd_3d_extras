use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;
use bevy_xpbd_3d_extras::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, PhysicsPlugins::default()));
    app.add_plugins(PhysicsDebugPlugin::default());

    app.add_plugins((FloatingBodyPlugin::default(), FloatingBodyDebugPlugin));

    app.add_systems(Startup, setup)
        .add_systems(Update, move_controller);

    app.run();
}

fn setup(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    // Floating Body
    commands.spawn((
        FloatingBody {
            float_height: 1.1,
            buffer_height: 0.45,

            // If shape is [`None`] then a raycast is used instead.
            // `0.39` is used so we can't detect a wall
            shape: Some(Collider::cylinder(0.0, 0.39)),
            ..default()
        },
        RigidBody::Dynamic,
        LockedAxes::ROTATION_LOCKED, // [`FloatingBody`] doesn't correct rotation
        Collider::capsule(0.45, 0.4),
        Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
        Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
        SpatialBundle {
            transform: Transform::from_xyz(0.0, 2.0, 0.0),
            ..default()
        },
    ));

    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 2.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Floor
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(20.0, 1.0, 20.0),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(20.0, 1.0, 20.0)),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, -1.0, 0.0),
            ..default()
        },
    ));

    // Step
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(5.0, 1.0, 5.0),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(5.0, 1.0, 5.0)),
            material: materials.add(StandardMaterial {
                base_color: Color::GOLD,
                ..default()
            }),
            transform: Transform::from_xyz(5.0, -0.6, 0.0),
            ..default()
        },
    ));

    // Capsule
    commands.spawn((
        RigidBody::Dynamic,
        LockedAxes::ALL_LOCKED.unlock_translation_y(),
        Collider::capsule(0.9, 0.4),
        SpatialBundle {
            transform: Transform::from_xyz(-1.0, 2.0, 0.0),
            ..default()
        },
    ));

    // Sensor
    commands.spawn((
        Sensor,
        Collider::cuboid(1.0, 1.0, 1.0),
        SpatialBundle {
            transform: Transform::from_xyz(0.0, 0.0, -3.0),
            ..default()
        },
    ));
}

fn move_controller(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut FloatingBody, &mut LinearVelocity, Has<Grounded>)>,
) {
    for (mut fb, mut lin_vel, is_grounded) in query.iter_mut() {
        lin_vel.x = (keys.pressed(KeyCode::KeyD) as i8 - keys.pressed(KeyCode::KeyA) as i8) as f32;
        lin_vel.z = (keys.pressed(KeyCode::KeyS) as i8 - keys.pressed(KeyCode::KeyW) as i8) as f32;

        lin_vel.x *= 4.0;
        lin_vel.z *= 4.0;

        if keys.just_pressed(KeyCode::Space) && is_grounded {
            lin_vel.y = 5.0;
            fb.enabled = false;
        } else if lin_vel.y < std::f32::EPSILON {
            fb.enabled = true;
        }
    }
}
