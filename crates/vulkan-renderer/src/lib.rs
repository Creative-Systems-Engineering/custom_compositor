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

pub use instance::VulkanInstance;
pub use device::VulkanDevice;
pub use swapchain::Swapchain;

/// Main Vulkan renderer context
pub struct VulkanRenderer {
    instance: VulkanInstance,
    device: VulkanDevice,
    swapchain: Option<Swapchain>,
}

impl VulkanRenderer {
    /// Create a new Vulkan renderer
    pub fn new() -> Result<Self> {
        let instance = VulkanInstance::new()?;
        let device = VulkanDevice::new(&instance)?;
        
        Ok(Self {
            instance,
            device,
            swapchain: None,
        })
    }
    
    /// Initialize swapchain for a given surface
    pub fn initialize_swapchain(&mut self, surface: ash::vk::SurfaceKHR, width: u32, height: u32) -> Result<()> {
        let swapchain = Swapchain::new(&self.instance, &self.device, surface, width, height)?;
        self.swapchain = Some(swapchain);
        Ok(())
    }
    
    /// Begin a frame for rendering
    pub fn begin_frame(&mut self) -> Result<()> {
        if let Some(ref mut swapchain) = self.swapchain {
            swapchain.acquire_next_image()?;
        }
        Ok(())
    }
    
    /// End frame and present
    pub fn end_frame(&mut self) -> Result<()> {
        if let Some(ref mut swapchain) = self.swapchain {
            swapchain.present()?;
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
        // Cleanup will be handled by individual component Drop implementations
        tracing::info!("Vulkan renderer shutting down");
    }
}
