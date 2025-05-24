// Vulkan Renderer - High-performance GPU-accelerated rendering
//
// This crate provides a complete Vulkan rendering pipeline optimized for
// 4K displays and modern graphics features including glassmorphism effects.

use compositor_utils::prelude::*;

pub mod instance;
pub mod device;
pub mod swapchain;
pub mod pipeline;
pub mod memory;
pub mod command;
pub mod sync;
pub mod surface;
pub mod buffer;
pub mod image;
pub mod descriptor;
pub mod surface_renderer;
pub mod surface_pipeline;
pub mod compositor_renderer;

pub use instance::VulkanInstance;
pub use device::VulkanDevice;
pub use swapchain::Swapchain;
pub use surface_renderer::{SurfaceRenderer, SurfaceTexture, SurfaceBuffer};
pub use surface_pipeline::{SurfacePipeline, SurfacePushConstants, SurfaceVertex};
pub use compositor_renderer::CompositorRenderer;

/// Main Vulkan renderer context
pub struct VulkanRenderer {
    instance: VulkanInstance,
    device: VulkanDevice,
    swapchain: Option<Swapchain>,
    compositor_renderer: Option<CompositorRenderer>,
}

impl VulkanRenderer {
    /// Create a new Vulkan renderer
    pub fn new() -> Result<Self> {
        let instance = VulkanInstance::new()?;
        let device = VulkanDevice::new(&instance)?;
        
        // Create compositor renderer for complete rendering pipeline
        let compositor_renderer = CompositorRenderer::new(instance.clone(), device.clone())?;
        
        Ok(Self {
            instance,
            device,
            swapchain: None,
            compositor_renderer: Some(compositor_renderer),
        })
    }
    
    /// Initialize swapchain for a given surface
    pub fn initialize_swapchain(&mut self, surface: ash::vk::SurfaceKHR, width: u32, height: u32) -> Result<()> {
        let swapchain = Swapchain::new(&self.instance, &self.device, surface, width, height)?;
        
        // Initialize compositor renderer with swapchain details
        if let (Some(ref mut compositor_renderer), Some(ref swapchain)) = 
            (&mut self.compositor_renderer, &self.swapchain) {
            compositor_renderer.initialize_swapchain(
                swapchain.images().to_vec(),
                swapchain.image_views().to_vec(),
                swapchain.extent(),
                swapchain.format(),
            )?;
        }
        
        self.swapchain = Some(swapchain);
        Ok(())
    }
    
    /// Begin a frame for rendering
    pub fn begin_frame(&mut self) -> Result<u32> {
        if let Some(ref mut swapchain) = self.swapchain {
            swapchain.acquire_next_image()
        } else {
            Err(CompositorError::runtime("Swapchain not initialized"))
        }
    }
    
    /// Render all surface textures to the screen
    pub fn render_frame(&mut self, frame_index: usize, image_index: u32) -> Result<ash::vk::CommandBuffer> {
        if let Some(ref mut compositor_renderer) = self.compositor_renderer {
            compositor_renderer.render_frame(frame_index, image_index)
        } else {
            Err(CompositorError::runtime("Compositor renderer not initialized"))
        }
    }
    
    /// Update a surface texture with new buffer data
    pub fn update_surface_buffer(
        &mut self, 
        surface_id: u32, 
        buffer_data: &[u8], 
        width: u32, 
        height: u32, 
        format: ash::vk::Format
    ) -> Result<()> {
        if let Some(ref mut compositor_renderer) = self.compositor_renderer {
            compositor_renderer.update_surface_texture(surface_id, buffer_data, width, height, format)?;
            debug!("Updated surface {} buffer ({}x{})", surface_id, width, height);
        }
        Ok(())
    }
    
    /// Update surface texture from Wayland client
    pub fn update_surface_texture(
        &mut self,
        surface_id: u32,
        buffer_data: &[u8],
        width: u32,
        height: u32,
        format: ash::vk::Format,
    ) -> Result<()> {
        if let Some(ref mut compositor_renderer) = self.compositor_renderer {
            compositor_renderer.update_surface_texture(surface_id, buffer_data, width, height, format)?;
        }
        Ok(())
    }

    /// Remove a surface texture
    pub fn remove_surface(&mut self, surface_id: u32) -> Result<()> {
        if let Some(ref mut compositor_renderer) = self.compositor_renderer {
            compositor_renderer.remove_surface(surface_id)?;
            debug!("Removed surface {}", surface_id);
        }
        Ok(())
    }
    
    /// End frame and present
    pub fn end_frame(&mut self) -> Result<()> {
        // Note: In a real implementation, frame_index and image_index would be tracked properly
        // For now, using placeholder values for compilation
        if let Some(ref mut compositor_renderer) = self.compositor_renderer {
            if let Some(ref mut swapchain) = self.swapchain {
                let image_index = swapchain.acquire_next_image()?;
                let _command_buffer = compositor_renderer.render_frame(0, image_index)?;
                
                // Present the frame
                swapchain.present()?;
            }
        }
        Ok(())
    }
    
    /// Get renderer information for debugging
    pub fn get_info(&self) -> RendererInfo {
        RendererInfo {
            api_version: self.instance.get_api_version(),
            device_name: self.device.get_device_name(),
            vendor_id: self.device.get_vendor_id(),
            device_type: self.device.get_device_type(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RendererInfo {
    pub api_version: u32,
    pub device_name: String,
    pub vendor_id: u32,
    pub device_type: String,
}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        // Cleanup compositor renderer first
        if let Some(compositor_renderer) = self.compositor_renderer.take() {
            drop(compositor_renderer);
        }
        
        // Cleanup will be handled by individual component Drop implementations
        tracing::info!("Vulkan renderer shutting down");
    }
}
