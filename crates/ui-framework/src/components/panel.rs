use compositor_utils::Result;
use glam::Vec2;

/// Panel component for UI framework
#[derive(Debug, Clone)]
pub struct Panel {
    pub position: Vec2,
    pub size: Vec2,
    pub background_color: [f32; 4], // RGBA
    pub border_width: f32,
    pub border_color: [f32; 4],
    pub is_visible: bool,
    pub opacity: f32,
}

impl Panel {
    /// Create a new panel
    pub fn new(position: Vec2, size: Vec2) -> Self {
        Self {
            position,
            size,
            background_color: [0.2, 0.2, 0.2, 0.8], // Dark transparent
            border_width: 1.0,
            border_color: [0.4, 0.4, 0.4, 1.0], // Gray border
            is_visible: true,
            opacity: 1.0,
        }
    }
    
    /// Set background color
    pub fn set_background_color(&mut self, color: [f32; 4]) {
        self.background_color = color;
    }
    
    /// Set border properties
    pub fn set_border(&mut self, width: f32, color: [f32; 4]) {
        self.border_width = width;
        self.border_color = color;
    }
    
    /// Set visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.is_visible = visible;
    }
    
    /// Set opacity
    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }
    
    /// Check if point is within panel bounds
    pub fn contains_point(&self, point: Vec2) -> bool {
        self.is_visible &&
        point.x >= self.position.x && 
        point.x <= self.position.x + self.size.x &&
        point.y >= self.position.y && 
        point.y <= self.position.y + self.size.y
    }
    
    /// Get the client area (excluding border)
    pub fn client_area(&self) -> (Vec2, Vec2) {
        let client_pos = Vec2::new(
            self.position.x + self.border_width,
            self.position.y + self.border_width
        );
        let client_size = Vec2::new(
            self.size.x - (self.border_width * 2.0),
            self.size.y - (self.border_width * 2.0)
        );
        (client_pos, client_size)
    }
    
    /// Update panel (called each frame)
    pub fn update(&mut self) -> Result<()> {
        // Update animations, effects, etc.
        Ok(())
    }
}

impl Default for Panel {
    fn default() -> Self {
        Self::new(Vec2::ZERO, Vec2::new(200.0, 150.0))
    }
}
