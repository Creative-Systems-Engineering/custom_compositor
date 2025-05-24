// App Bar - Side-docked app bar (flagship feature)
//
// This crate implements the custom side-docked app bar that remains
// always on top and provides quick access to applications and system controls.

use compositor_utils::prelude::*;

pub mod dock;
pub mod launcher;
pub mod widgets;
pub mod config;

/// Main app bar component
pub struct AppBar {
    position: AppBarPosition,
    size: AppBarSize,
}

#[derive(Debug, Clone, Copy)]
pub enum AppBarPosition {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy)]
pub struct AppBarSize {
    pub width: f32,
    pub height: f32,
}

impl AppBar {
    /// Create a new app bar
    pub fn new() -> Result<Self> {
        info!("Initializing App Bar");
        
        Ok(Self {
            position: AppBarPosition::Left,
            size: AppBarSize { width: 80.0, height: 1080.0 },
        })
    }
}

impl Default for AppBar {
    fn default() -> Self {
        Self::new().expect("Failed to create app bar")
    }
}
