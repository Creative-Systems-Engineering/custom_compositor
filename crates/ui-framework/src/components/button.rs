use compositor_utils::Result;
use glam::Vec2;

/// Button component for UI framework
#[derive(Debug, Clone)]
pub struct Button {
    pub position: Vec2,
    pub size: Vec2,
    pub text: String,
    pub is_pressed: bool,
    pub is_hovered: bool,
    pub is_enabled: bool,
}

impl Button {
    /// Create a new button
    pub fn new(text: String, position: Vec2, size: Vec2) -> Self {
        Self {
            position,
            size,
            text,
            is_pressed: false,
            is_hovered: false,
            is_enabled: true,
        }
    }
    
    /// Handle mouse press event
    pub fn on_press(&mut self, mouse_pos: Vec2) -> bool {
        if self.contains_point(mouse_pos) && self.is_enabled {
            self.is_pressed = true;
            true
        } else {
            false
        }
    }
    
    /// Handle mouse release event
    pub fn on_release(&mut self, mouse_pos: Vec2) -> bool {
        if self.is_pressed && self.contains_point(mouse_pos) {
            self.is_pressed = false;
            true // Button was clicked
        } else {
            self.is_pressed = false;
            false
        }
    }
    
    /// Handle mouse hover event
    pub fn on_hover(&mut self, mouse_pos: Vec2) {
        self.is_hovered = self.contains_point(mouse_pos);
    }
    
    /// Check if point is within button bounds
    pub fn contains_point(&self, point: Vec2) -> bool {
        point.x >= self.position.x && 
        point.x <= self.position.x + self.size.x &&
        point.y >= self.position.y && 
        point.y <= self.position.y + self.size.y
    }
    
    /// Set button enabled state
    pub fn set_enabled(&mut self, enabled: bool) {
        self.is_enabled = enabled;
        if !enabled {
            self.is_pressed = false;
            self.is_hovered = false;
        }
    }
    
    /// Update button (called each frame)
    pub fn update(&mut self) -> Result<()> {
        // Update animations, state, etc.
        Ok(())
    }
}

impl Default for Button {
    fn default() -> Self {
        Self::new("Button".to_string(), Vec2::ZERO, Vec2::new(100.0, 30.0))
    }
}
