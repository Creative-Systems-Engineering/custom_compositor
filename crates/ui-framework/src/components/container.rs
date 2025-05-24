use compositor_utils::Result;
use glam::Vec2;

/// Layout direction for container
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutDirection {
    Horizontal,
    Vertical,
}

/// Container component for UI framework
#[derive(Debug, Clone)]
pub struct Container {
    pub position: Vec2,
    pub size: Vec2,
    pub padding: f32,
    pub spacing: f32,
    pub layout_direction: LayoutDirection,
    pub background_color: Option<[f32; 4]>,
    pub is_visible: bool,
    pub children_count: usize, // Simplified - in real implementation would hold child widgets
}

impl Container {
    /// Create a new container
    pub fn new(position: Vec2, size: Vec2) -> Self {
        Self {
            position,
            size,
            padding: 8.0,
            spacing: 4.0,
            layout_direction: LayoutDirection::Vertical,
            background_color: None,
            is_visible: true,
            children_count: 0,
        }
    }
    
    /// Set padding around children
    pub fn set_padding(&mut self, padding: f32) {
        self.padding = padding.max(0.0);
    }
    
    /// Set spacing between children
    pub fn set_spacing(&mut self, spacing: f32) {
        self.spacing = spacing.max(0.0);
    }
    
    /// Set layout direction
    pub fn set_layout_direction(&mut self, direction: LayoutDirection) {
        self.layout_direction = direction;
    }
    
    /// Set background color (None for transparent)
    pub fn set_background_color(&mut self, color: Option<[f32; 4]>) {
        self.background_color = color;
    }
    
    /// Set visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.is_visible = visible;
    }
    
    /// Get the client area (excluding padding)
    pub fn client_area(&self) -> (Vec2, Vec2) {
        let client_pos = Vec2::new(
            self.position.x + self.padding,
            self.position.y + self.padding
        );
        let client_size = Vec2::new(
            self.size.x - (self.padding * 2.0),
            self.size.y - (self.padding * 2.0)
        );
        (client_pos, client_size)
    }
    
    /// Calculate child position based on index and layout
    pub fn child_position(&self, child_index: usize, child_size: Vec2) -> Vec2 {
        let (client_pos, _client_size) = self.client_area();
        
        match self.layout_direction {
            LayoutDirection::Horizontal => {
                Vec2::new(
                    client_pos.x + (child_index as f32 * (child_size.x + self.spacing)),
                    client_pos.y
                )
            }
            LayoutDirection::Vertical => {
                Vec2::new(
                    client_pos.x,
                    client_pos.y + (child_index as f32 * (child_size.y + self.spacing))
                )
            }
        }
    }
    
    /// Check if point is within container bounds
    pub fn contains_point(&self, point: Vec2) -> bool {
        self.is_visible &&
        point.x >= self.position.x && 
        point.x <= self.position.x + self.size.x &&
        point.y >= self.position.y && 
        point.y <= self.position.y + self.size.y
    }
    
    /// Add a child (simplified - just increment count)
    pub fn add_child(&mut self) {
        self.children_count += 1;
    }
    
    /// Remove a child (simplified - just decrement count)
    pub fn remove_child(&mut self) {
        if self.children_count > 0 {
            self.children_count -= 1;
        }
    }
    
    /// Update container (called each frame)
    pub fn update(&mut self) -> Result<()> {
        // Update layout, animations, etc.
        Ok(())
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new(Vec2::ZERO, Vec2::new(300.0, 200.0))
    }
}
