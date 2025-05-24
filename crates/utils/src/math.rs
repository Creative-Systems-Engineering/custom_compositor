use glam::{Mat4, Vec2};

/// 4K resolution constants
pub const UHD_WIDTH: u32 = 3840;
pub const UHD_HEIGHT: u32 = 2160;
pub const UHD_ASPECT_RATIO: f32 = UHD_WIDTH as f32 / UHD_HEIGHT as f32;

/// Common DPI values for 4K displays
pub const DPI_96: f32 = 96.0;
pub const DPI_144: f32 = 144.0;
pub const DPI_192: f32 = 192.0;

/// Rectangle structure for UI layout
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
    
    pub fn from_size(width: f32, height: f32) -> Self {
        Self::new(0.0, 0.0, width, height)
    }
    
    pub fn center(&self) -> Vec2 {
        Vec2::new(self.x + self.width * 0.5, self.y + self.height * 0.5)
    }
    
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.x 
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }
    
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }
}

/// Create an orthographic projection matrix for 2D rendering
pub fn create_2d_projection(width: f32, height: f32) -> Mat4 {
    Mat4::orthographic_rh(0.0, width, height, 0.0, -1.0, 1.0)
}

/// Create a perspective projection matrix for 3D rendering
pub fn create_3d_projection(width: f32, height: f32, fov: f32, near: f32, far: f32) -> Mat4 {
    let aspect = width / height;
    Mat4::perspective_rh(fov, aspect, near, far)
}

/// Convert screen coordinates to normalized device coordinates
pub fn screen_to_ndc(screen_pos: Vec2, screen_size: Vec2) -> Vec2 {
    Vec2::new(
        (screen_pos.x / screen_size.x) * 2.0 - 1.0,
        (screen_pos.y / screen_size.y) * 2.0 - 1.0,
    )
}

/// Convert normalized device coordinates to screen coordinates
pub fn ndc_to_screen(ndc_pos: Vec2, screen_size: Vec2) -> Vec2 {
    Vec2::new(
        (ndc_pos.x + 1.0) * 0.5 * screen_size.x,
        (ndc_pos.y + 1.0) * 0.5 * screen_size.y,
    )
}

/// Calculate DPI scaling factor
pub fn calculate_dpi_scale(dpi: f32) -> f32 {
    dpi / DPI_96
}
