use bevy::{prelude::*, transform::TransformSystem};
use bevy_xpbd_3d::prelude::*;
use bevy_xpbd_3d_extras::{prelude::*, vehicle::Wheel};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            VehicleBodyPlugin::default(),
        ))
        .add_plugins((PhysicsDebugPlugin::default(), VehicleBodyDebugPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, move_controller)
        .add_systems(
            PostUpdate,
            follow_vehicle
                .after(PhysicsSet::Sync)
                .before(TransformSystem::TransformPropagate),
        )
        .run()
}

fn setup(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    // Vehicle
    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 0.5, 2.0),
        VehicleBody {
            wheels: vec![
                Wheel {
                    offset: Vec3::new(0.45, 0.0, -0.975),
                    ..default()
                },
                Wheel {
                    offset: Vec3::new(-0.45, 0.0, -0.975),
                    ..default()
                },
                Wheel {
                    offset: Vec3::new(0.45, 0.0, 0.975),
                    ..default()
                },
                Wheel {
                    offset: Vec3::new(-0.45, 0.0, 0.975),
                    ..default()
                },
            ],
        },
        SpatialBundle {
            transform: Transform::from_xyz(0.0, 10.0, 0.0),
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
        Collider::cuboid(10000.0, 1.0, 10000.0),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(10000.0, 1.0, 10000.0)),
            material: materials.add(Color::rgb(1.0, 1.0, 1.0)),
            transform: Transform::from_xyz(0.0, -0.5, 0.0),
            ..default()
        },
    ));

    // Step
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(10.0, 1.0, 10.0),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(10.0, 1.0, 10.0)),
            material: materials.add(Color::rgb(1.0, 0.5, 0.5)),
            transform: Transform::from_xyz(0.0, -0.3, -15.0),
            ..default()
        },
    ));
}

fn move_controller(keys: Res<ButtonInput<KeyCode>>, mut query: Query<&mut VehicleBody>) {
    let forward = keys.pressed(KeyCode::KeyS) as i8 - keys.pressed(KeyCode::KeyW) as i8;
    let angle = keys.pressed(KeyCode::KeyA) as i8 - keys.pressed(KeyCode::KeyD) as i8;
    let brake = keys.pressed(KeyCode::Space) as i8;

    for mut vb in query.iter_mut() {
        for wheel in &mut vb.wheels[0..=1] {
            wheel.angle = wheel.angle.lerp(angle as f32 * 45f32.to_radians(), 0.1);
        }

        for wheel in &mut vb.wheels[2..=3] {
            wheel.acceleration = forward as f32 * 8.0;
            wheel.brake = brake as f32 * 20.0;

            if forward == 0 && brake == 0 {
                wheel.brake = 0.1;
            }
        }
    }
}

fn follow_vehicle(
    vehicle_query: Query<Entity, With<VehicleBody>>,
    mut query: ParamSet<(TransformHelper, Query<&mut Transform, With<Camera3d>>)>,
) {
    let vehicle = vehicle_query.single();
    let Ok(global_transform) = query.p0().compute_global_transform(vehicle) else {
        return;
    };

    let mut p1 = query.p1();
    let mut camera = p1.single_mut();

    camera.translation = global_transform.translation() - global_transform.forward() * 3.0
        + global_transform.up() * 1.5;
    camera.rotation = global_transform.to_scale_rotation_translation().1
        * Quat::from_axis_angle(Vec3::X, -20f32.to_radians());
}
