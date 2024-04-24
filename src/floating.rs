use bevy::{ecs::schedule::ScheduleLabel, prelude::*, utils::intern::Interned};
use bevy_xpbd_3d::prelude::*;

use crate::prelude::{Grounded, UpDirection};

// --- Components ----
/// Handles: Grounded, Slopes, Stairs, Snap to Ground
#[derive(Component)]
pub struct FloatingBody {
    /// Enables adding counter force to float the [`RigidBody`]
    pub enabled: bool,

    /// The offset of the ray relative to the transform of it's entity
    pub ray_offset: Vec3,
    pub float_height: f32,
    pub buffer_height: f32,
    pub shape: Option<Collider>,

    pub damp_frequency: f32,
    pub damp_factor: f32,
}

impl Default for FloatingBody {
    fn default() -> Self {
        Self {
            enabled: true,
            ray_offset: Vec3::ZERO,
            float_height: 1.0,
            buffer_height: 1.0,
            shape: None,

            damp_frequency: 15.0,
            damp_factor: 0.3,
        }
    }
}

// ---- Systems ----
fn solve_constraint(
    time: Res<Time>,
    spatial_query: Res<SpatialQueryPipeline>,
    hit_query: Query<(&LinearVelocity, Has<RigidBody>, Has<Sensor>), Without<FloatingBody>>,
    mut query: Query<(
        Entity,
        &Position,
        &FloatingBody,
        Option<&UpDirection>,
        &mut LinearVelocity,
        Has<Grounded>,
    )>,
    mut commands: Commands,
) {
    for (entity, pos, fb, up, mut lin_vel, grounded) in query.iter_mut() {
        let up_dir = up.map(|d| d.0).unwrap_or(Direction3d::Y);
        let up_vec = Into::<Vec3>::into(up_dir);

        let max_toi = fb.float_height + fb.buffer_height;
        let hit: Option<(Entity, f32)> = if let Some(shape) = &fb.shape {
            let hits = spatial_query.shape_hits(
                shape,
                pos.0 + fb.ray_offset,
                Quat::IDENTITY,
                -up_dir,
                max_toi,
                8,
                false,
                SpatialQueryFilter::from_excluded_entities([entity]),
            );

            let hit = hits
                .into_iter()
                .filter(|d| {
                    hit_query
                        .get(d.entity)
                        .is_ok_and(|(_, rb, sensor)| rb && !sensor)
                        && d.time_of_impact.is_finite()
                })
                .min_by(|a, b| a.time_of_impact.total_cmp(&b.time_of_impact));

            hit.map(|d| (d.entity, d.time_of_impact))
        } else {
            let hits = spatial_query.ray_hits(
                pos.0 + fb.ray_offset,
                -up_dir,
                max_toi,
                8,
                true,
                SpatialQueryFilter::from_excluded_entities([entity]),
            );

            let hit = hits
                .into_iter()
                .filter(|d| {
                    hit_query
                        .get(d.entity)
                        .is_ok_and(|(_, rb, sensor)| rb && !sensor)
                        && d.time_of_impact.is_finite()
                })
                .min_by(|a, b| a.time_of_impact.total_cmp(&b.time_of_impact));

            hit.map(|d| (d.entity, d.time_of_impact))
        };

        if let Some((hit_entity, hit_toi)) = hit {
            let Ok((hit_vel, _, _)) = hit_query.get(hit_entity) else {
                continue;
            };

            let rel_vel = lin_vel.dot(-up_vec) - hit_vel.dot(-up_vec);
            let x = hit_toi - fb.float_height;

            if fb.enabled && !(x > f32::EPSILON && !grounded) {
                let tension = x * (fb.damp_frequency * fb.damp_frequency);
                let damp = rel_vel * (fb.damp_factor * (2.0 * fb.damp_frequency));
                lin_vel.0 -= (tension - damp) * up_vec * time.delta_seconds();

                // TODO: Add a small force to the hit entity?
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

        let float_end = pos.0 + fb.ray_offset + -up_vec * fb.float_height;
        gizmos.line(pos.0 + fb.ray_offset, float_end, Color::RED);
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
