use bevy::prelude::*;

/// An [`Entity`] with this component is grounded
#[derive(Component)]
pub struct Grounded;

/// The up direction of the [`RigidBody`].  
/// Without this, [`Direction3d::Y`] is assumed.
#[derive(Component)]
pub struct UpDirection(pub Direction3d);
