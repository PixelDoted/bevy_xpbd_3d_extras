use bevy::{ecs::schedule::ScheduleLabel, prelude::*, utils::intern::Interned};
use bevy_xpbd_3d::prelude::*;

use crate::prelude::{Grounded, UpDirection};

// --- Components ----
/// Handles: Grounded, Slopes, Stairs, Snap to Ground
#[derive(Component)]
pub struct FloatingBody {
    /// Enables adding counter force to float the [`RigidBody`]
    pub enabled: bool,

    pub float_height: f32,
    pub buffer_height: f32,

    pub damp_frequency: f32,
    pub damp_factor: f32,
}

impl Default for FloatingBody {
    fn default() -> Self {
        Self {
            enabled: true,
            float_height: 1.0,
            buffer_height: 1.0,

            damp_frequency: 15.0,
            damp_factor: 0.3,
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
        &FloatingBody,
        Option<&UpDirection>,
        &mut LinearVelocity,
    )>,
    mut commands: Commands,
) {
    for (entity, pos, fb, up, mut lin_vel) in query.iter_mut() {
        let up_dir = up.map(|d| d.0).unwrap_or(Direction3d::Y);
        let up_vec = Into::<Vec3>::into(up_dir);

        let max_toi = fb.float_height + fb.buffer_height;
        let hit = spatial_query.cast_ray(
            pos.0,
            -up_dir,
            max_toi,
            true,
            SpatialQueryFilter::from_excluded_entities([entity]),
        );

        if let Some(hit) = hit {
            let rel_vel = lin_vel.dot(-up_vec);
            let x = hit.time_of_impact - fb.float_height;

            if fb.enabled {
                let tension = x * (fb.damp_frequency * fb.damp_frequency);
                let damp = rel_vel * (fb.damp_factor * (2.0 * fb.damp_frequency));
                lin_vel.0 -= (tension - damp) * up_vec * time.delta_seconds();
            }

            if x < f32::EPSILON {
                commands.entity(entity).insert(Grounded);
            }
        } else {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}

fn debug_gizmos(
    query: Query<(&Position, &FloatingBody, Option<&UpDirection>)>,
    mut gizmos: Gizmos,
) {
    for (pos, fb, up) in query.iter() {
        let up_dir = up.map(|d| d.0).unwrap_or(Direction3d::Y);
        let up_vec = Into::<Vec3>::into(up_dir);

        let float_end = pos.0 + -up_vec * fb.float_height;
        gizmos.line(pos.0, float_end, Color::RED);
        gizmos.line(
            float_end,
            float_end + -up_vec * fb.buffer_height,
            Color::BLUE,
        );
    }
}

// ---- Plugins ----
pub struct FloatingBodyPlugin {
    schedule_label: Interned<dyn ScheduleLabel>,
}

impl FloatingBodyPlugin {
    pub fn new(schedule: impl ScheduleLabel) -> Self {
        Self {
            schedule_label: schedule.intern(),
        }
    }
}

impl Default for FloatingBodyPlugin {
    fn default() -> Self {
        Self::new(PostUpdate)
    }
}

impl Plugin for FloatingBodyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            self.schedule_label,
            solve_constraint
                .before(PhysicsSet::StepSimulation)
                .after(PhysicsSet::Prepare),
        );
    }
}

pub struct FloatingBodyDebugPlugin;

impl Plugin for FloatingBodyDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, debug_gizmos);
    }
}
