use compositor_utils::Result;
use glam::Vec2;

/// Text alignment options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

/// Text component for UI framework
#[derive(Debug, Clone)]
pub struct Text {
    pub position: Vec2,
    pub content: String,
    pub font_size: f32,
    pub color: [f32; 4], // RGBA
    pub alignment: TextAlign,
    pub max_width: Option<f32>,
    pub is_visible: bool,
    pub line_height: f32,
}

impl Text {
    /// Create a new text component
    pub fn new(content: String, position: Vec2) -> Self {
        Self {
            position,
            content,
            font_size: 16.0,
            color: [1.0, 1.0, 1.0, 1.0], // White
            alignment: TextAlign::Left,
            max_width: None,
            is_visible: true,
            line_height: 1.2,
        }
    }
    
    /// Set font size
    pub fn set_font_size(&mut self, size: f32) {
        self.font_size = size.max(1.0);
    }
    
    /// Set text color
    pub fn set_color(&mut self, color: [f32; 4]) {
        self.color = color;
    }
    
    /// Set text alignment
    pub fn set_alignment(&mut self, alignment: TextAlign) {
        self.alignment = alignment;
    }
    
    /// Set maximum width for text wrapping
    pub fn set_max_width(&mut self, width: Option<f32>) {
        self.max_width = width;
    }
    
    /// Set visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.is_visible = visible;
    }
    
    /// Set line height multiplier
    pub fn set_line_height(&mut self, height: f32) {
        self.line_height = height.max(0.1);
    }
    
    /// Update text content
    pub fn set_content(&mut self, content: String) {
        self.content = content;
    }
    
    /// Get estimated text bounds (simplified calculation)
    pub fn estimated_bounds(&self) -> Vec2 {
        if !self.is_visible || self.content.is_empty() {
            return Vec2::ZERO;
        }
        
        // Simplified text metrics - in a real implementation,
        // this would use proper font metrics
        let char_width = self.font_size * 0.6; // Approximate
        let line_height = self.font_size * self.line_height;
        
        if let Some(max_width) = self.max_width {
            let chars_per_line = (max_width / char_width) as usize;
            let lines = (self.content.len() + chars_per_line - 1) / chars_per_line.max(1);
            Vec2::new(max_width, line_height * lines as f32)
        } else {
            Vec2::new(char_width * self.content.len() as f32, line_height)
        }
    }
    
    /// Update text (called each frame)
    pub fn update(&mut self) -> Result<()> {
        // Update animations, effects, etc.
        Ok(())
    }
}

impl Default for Text {
    fn default() -> Self {
        Self::new("Text".to_string(), Vec2::ZERO)
    }
}
