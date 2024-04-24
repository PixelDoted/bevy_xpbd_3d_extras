pub mod components;
#[cfg(feature = "floating")]
pub mod floating;
#[cfg(feature = "vehicle")]
pub mod vehicle;

pub mod prelude {
    pub use crate::bodies::*;
    pub use crate::components::*;
}

pub mod bodies {
    #[cfg(feature = "floating")]
    pub use crate::floating::{FloatingBody, FloatingBodyDebugPlugin, FloatingBodyPlugin};
    #[cfg(feature = "vehicle")]
    pub use crate::vehicle::{VehicleBody, VehicleBodyDebugPlugin, VehicleBodyPlugin};
}
