pub mod components;
pub mod floating;

pub mod prelude {
    pub use crate::bodies::*;
    pub use crate::components::*;
}

pub mod bodies {
    pub use crate::floating::{FloatingBody, FloatingBodyDebugPlugin, FloatingBodyPlugin};
}
