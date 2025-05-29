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

#[cfg(test)]
mod tests;

pub use instance::VulkanInstance;
pub use device::VulkanDevice;
pub use swapchain::Swapchain;
pub use surface_renderer::{SurfaceRenderer, SurfaceTexture, SurfaceBuffer};
pub use surface_pipeline::{SurfacePipeline, SurfacePushConstants, SurfaceVertex};
pub use compositor_renderer::CompositorRenderer;

/// Main Vulkan renderer context
pub struct VulkanRenderer {
    instance: Option<VulkanInstance>,
    device: Option<VulkanDevice>,
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
            instance: Some(instance),
            device: Some(device),
            swapchain: None,
            compositor_renderer: Some(compositor_renderer),
        })
    }
    
    /// Initialize swapchain for a given surface
    pub fn initialize_swapchain(&mut self, surface: ash::vk::SurfaceKHR, width: u32, height: u32) -> Result<()> {
        let (instance, device) = match (&self.instance, &self.device) {
            (Some(instance), Some(device)) => (instance, device),
            _ => return Err(CompositorError::runtime("Vulkan instance or device not available")),
        };
        
        let swapchain = Swapchain::new(instance, device, surface, width, height)?;
        
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
        let (instance, device) = match (&self.instance, &self.device) {
            (Some(instance), Some(device)) => (instance, device),
            _ => return RendererInfo {
                api_version: 0,
                device_name: "Not Available".to_string(),
                vendor_id: 0,
                device_type: "Unknown".to_string(),
            },
        };
        
        RendererInfo {
            api_version: instance.api_version(),
            device_name: device.get_device_name(),
            vendor_id: device.get_vendor_id(),
            device_type: device.get_device_type(),
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
        tracing::info!("Starting Vulkan renderer cleanup...");
        
        // CRITICAL: Wait for device to be idle before destroying anything
        if let Some(ref device) = self.device {
            if let Err(e) = device.wait_idle() {
                tracing::error!("Failed to wait for device idle during cleanup: {}", e);
            }
        }
        
        // Destroy in reverse order of creation:
        // 1. High-level renderer (contains command pools, pipelines, etc.)
        if let Some(compositor_renderer) = self.compositor_renderer.take() {
            tracing::info!("Destroying compositor renderer...");
            drop(compositor_renderer);
        }
        
        // 2. Swapchain (contains images, image views, framebuffers)
        if let Some(swapchain) = self.swapchain.take() {
            tracing::info!("Destroying swapchain...");
            drop(swapchain);
        }
        
        // 3. Device (automatically destroys remaining device objects)
        if let Some(device) = self.device.take() {
            tracing::info!("Destroying Vulkan device...");
            drop(device);
        }
        
        // 4. Instance (last to be destroyed)
        if let Some(instance) = self.instance.take() {
            tracing::info!("Destroying Vulkan instance...");
            drop(instance);
        }
        
        tracing::info!("Vulkan renderer cleanup complete");
    }
}
