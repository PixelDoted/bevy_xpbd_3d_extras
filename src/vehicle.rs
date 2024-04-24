use bevy::{ecs::schedule::ScheduleLabel, prelude::*, utils::intern::Interned};
use bevy_xpbd_3d::prelude::*;

// ---- Components ----
#[derive(Component)]
pub struct VehicleBody {
    pub wheels: Vec<Wheel>,
}

pub struct Wheel {
    /// The wheels offset, relative to it's entity
    pub offset: Vec3,

    /// The length of the spring
    pub length: f32,
    pub mass: f32,

    pub grip: f32,
    pub acceleration: f32,
    pub brake: f32,
    pub angle: f32,

    pub up: Direction3d,
    pub forward: Direction3d,

    pub spring_strength: f32,
    pub spring_damping: f32,
}

impl Default for Wheel {
    fn default() -> Self {
        Self {
            offset: Vec3::ZERO,
            length: 0.8,
            mass: 0.5,
            grip: 1.0,
            acceleration: 0.0,
            brake: 0.0,
            angle: 0.0,
            up: Direction3d::Y,
            forward: Direction3d::NEG_Z,
            spring_strength: 64.0,
            spring_damping: 16.0,
        }
    }
}

// ---- Systems ----
fn solve_constraint(
    time: Res<Time>,
    spatial_query: Res<SpatialQueryPipeline>,
    mut query: Query<(
        Entity,
        &Position,
        &Rotation,
        &VehicleBody,
        &mut LinearVelocity,
        &mut AngularVelocity,
    )>,
) {
    for (entity, pos, rot, vb, mut lin_vel, mut ang_vel) in query.iter_mut() {
        let mut linear_force = Vec3::ZERO;
        let mut angular_force = Vec3::ZERO;

        for wheel in &vb.wheels {
            let origin = pos.0 + rot.0 * wheel.offset;
            let hits = spatial_query.ray_hits(
                origin,
                rot.0 * -wheel.up,
                wheel.length,
                8,
                true,
                SpatialQueryFilter::from_excluded_entities([entity]),
            );

            let hit = hits
                .into_iter()
                .min_by(|a, b| a.time_of_impact.total_cmp(&b.time_of_impact));

            if let Some(hit) = hit {
                let up_vec = Into::<Vec3>::into(rot.0 * wheel.up);
                let wheel_quat = Quat::from_axis_angle(up_vec, wheel.angle);
                let fwd_vec = wheel_quat * Into::<Vec3>::into(rot.0 * wheel.forward);
                let slip_vec = -up_vec.cross(fwd_vec);

                let x = wheel.length - hit.time_of_impact;
                let wheel_center = wheel.offset - up_vec * hit.time_of_impact;

                let point_vel = ang_vel.0 * wheel_center.length() + lin_vel.0;
                let up_vel = point_vel.dot(up_vec);
                let fwd_vel = lin_vel.0.dot(fwd_vec);
                let slip_vel = point_vel.dot(slip_vec);

                let suspension_force = x * wheel.spring_strength - up_vel * wheel.spring_damping;
                let slip_force = slip_vel * wheel.grip / time.delta_seconds() * wheel.mass;
                let accel_force = wheel.acceleration;
                let brake_force = fwd_vel * wheel.brake;

                let force = suspension_force * -up_vec
                    + slip_force * slip_vec
                    + accel_force * fwd_vec
                    + brake_force * fwd_vec;
                linear_force -= force;
                angular_force -= wheel_center.cross(force);
            }
        }

        linear_force /= vb.wheels.len() as f32;
        angular_force /= vb.wheels.len() as f32;
        if linear_force.is_finite() {
            lin_vel.0 += linear_force * time.delta_seconds();
        }
        if angular_force.is_finite() {
            ang_vel.0 += angular_force * time.delta_seconds();
        }

        let damping_force = ang_vel.0 * 8.0 * time.delta_seconds();
        ang_vel.0 -= damping_force;
    }
}

fn debug_gizmos(
    query: Query<(
        &Position,
        &Rotation,
        &LinearVelocity,
        &AngularVelocity,
        &VehicleBody,
    )>,
    mut gizmos: Gizmos,
) {
    for (pos, rot, lin_vel, ang_vel, vb) in query.iter() {
        for wheel in &vb.wheels {
            let up_vec = Into::<Vec3>::into(rot.0 * wheel.up);
            let wheel_quat = Quat::from_axis_angle(up_vec, wheel.angle);
            let fwd_vec = wheel_quat * Into::<Vec3>::into(rot.0 * wheel.forward);
            let slip_vec = -up_vec.cross(fwd_vec);

            let point_vel = ang_vel.0 * wheel.offset.length() + lin_vel.0;
            let up_vel = point_vel.dot(up_vec);
            let fwd_vel = lin_vel.0.dot(fwd_vec);
            let slip_vel = point_vel.dot(slip_vec);

            let pos = pos.0 + rot.0 * wheel.offset;
            gizmos.arrow(pos, pos + slip_vel * slip_vec, Color::RED);
            gizmos.arrow(pos, pos + up_vel * up_vec, Color::GREEN);
            gizmos.arrow(pos, pos + fwd_vel * fwd_vec, Color::BLUE);
            gizmos.line(pos, pos - up_vec * wheel.length, Color::BLACK);

            gizmos.arrow(
                pos,
                pos + (up_vel * up_vec + fwd_vel * fwd_vec + slip_vel * slip_vec),
                Color::PURPLE,
            );
        }
    }
}

// ---- Plugins ----
pub struct VehicleBodyPlugin {
    schedule_label: Interned<dyn ScheduleLabel>,
}

impl VehicleBodyPlugin {
    pub fn new(schedule: &'static impl ScheduleLabel) -> Self {
        Self {
            schedule_label: Interned(schedule),
        }
    }
}

impl Default for VehicleBodyPlugin {
    fn default() -> Self {
        Self::new(&PostUpdate)
    }
}

impl Plugin for VehicleBodyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            self.schedule_label,
            solve_constraint
                .before(PhysicsSet::StepSimulation)
                .after(PhysicsSet::Prepare),
        );
    }
}

pub struct VehicleBodyDebugPlugin;

impl Plugin for VehicleBodyDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, debug_gizmos.after(PhysicsSet::Sync));
    }
}
