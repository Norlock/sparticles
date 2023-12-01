pub mod color_animation;
pub mod force_animation;
pub mod gravity_animation;
pub mod stray_animation;

pub use color_animation::{ColorAnimation, ColorUniform, RegisterColorAnimation};
pub use force_animation::{ForceAnimation, ForceUniform, RegisterForceAnimation};
pub use gravity_animation::{
    GravityAnimation, GravityUniform, GravityUniformOptions, RegisterGravityAnimation,
};
pub use stray_animation::{RegisterStrayAnimation, StrayAnimation, StrayUniform};
