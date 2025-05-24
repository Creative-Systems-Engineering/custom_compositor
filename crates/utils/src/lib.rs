// Utils crate - Shared utilities and common functionality
//
// This crate provides common utilities, error types, logging setup,
// and shared functionality used across the entire compositor project.

pub mod error;
pub mod logging;
pub mod math;
pub mod memory;
pub mod async_utils;

// Re-export commonly used types
pub use error::{CompositorError, Result};
pub use logging::setup_logging;

/// Common prelude for the compositor project
pub mod prelude {
    pub use crate::error::{CompositorError, Result};
    pub use tracing::{debug, error, info, trace, warn};
    pub use anyhow::Context;
    pub use glam::{Vec2, Vec3, Vec4, Mat4};
}
