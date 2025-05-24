use ash::vk;
use compositor_utils::prelude::*;
use crate::{instance::VulkanInstance, device::VulkanDevice};

/// Vulkan swapchain wrapper for presenting rendered frames
pub struct Swapchain {
    swapchain_loader: ash::extensions::khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    #[allow(dead_code)] // Will be used for render pass operations and resource binding
    images: Vec<vk::Image>,
    #[allow(dead_code)] // Will be used for framebuffer creation and rendering
    image_views: Vec<vk::ImageView>,
    format: vk::Format,
    extent: vk::Extent2D,
    current_image: u32,
}

impl Swapchain {
    /// Create a new swapchain
    pub fn new(
        instance: &VulkanInstance,
        device: &VulkanDevice,
        surface: vk::SurfaceKHR,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let swapchain_loader = ash::extensions::khr::Swapchain::new(instance.handle(), device.handle());
        
        // Query surface capabilities
        let surface_loader = ash::extensions::khr::Surface::new(instance.entry(), instance.handle());
        let capabilities = unsafe {
            surface_loader.get_physical_device_surface_capabilities(device.physical_device(), surface)?
        };
        
        // Choose surface format
        let formats = unsafe {
            surface_loader.get_physical_device_surface_formats(device.physical_device(), surface)?
        };
        
        let format = Self::choose_surface_format(&formats);
        
        // Choose present mode (prefer mailbox for low latency)
        let present_modes = unsafe {
            surface_loader.get_physical_device_surface_present_modes(device.physical_device(), surface)?
        };
        
        let present_mode = Self::choose_present_mode(&present_modes);
        
        // Choose extent
        let extent = Self::choose_extent(&capabilities, width, height);
        
        // Image count (prefer triple buffering)
        let mut image_count = capabilities.min_image_count + 1;
        if capabilities.max_image_count > 0 && image_count > capabilities.max_image_count {
            image_count = capabilities.max_image_count;
        }
        
        // Create swapchain
        let swapchain_create_info = vk::SwapchainCreateInfoKHR {
            surface,
            min_image_count: image_count,
            image_format: format.format,
            image_color_space: format.color_space,
            image_extent: extent,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode: vk::SharingMode::EXCLUSIVE,
            pre_transform: capabilities.current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode,
            clipped: vk::TRUE,
            ..Default::default()
        };
        
        let swapchain = unsafe {
            swapchain_loader.create_swapchain(&swapchain_create_info, None)?
        };
        
        // Get swapchain images
        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };
        
        // Create image views
        let image_views = Self::create_image_views(device, &images, format.format)?;
        
        info!("Swapchain created: {}x{}, {} images", extent.width, extent.height, images.len());
        
        Ok(Self {
            swapchain_loader,
            swapchain,
            images,
            image_views,
            format: format.format,
            extent,
            current_image: 0,
        })
    }
    
    fn choose_surface_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
        // Prefer SRGB format for better color accuracy
        for format in formats {
            if format.format == vk::Format::B8G8R8A8_SRGB 
                && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR 
            {
                return *format;
            }
        }
        
        // Fallback to first available
        formats[0]
    }
    
    fn choose_present_mode(present_modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
        // Prefer mailbox for low latency and smooth rendering
        for &mode in present_modes {
            if mode == vk::PresentModeKHR::MAILBOX {
                return mode;
            }
        }
        
        // Fallback to FIFO (always available)
        vk::PresentModeKHR::FIFO
    }
    
    fn choose_extent(capabilities: &vk::SurfaceCapabilitiesKHR, width: u32, height: u32) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D {
                width: width.clamp(capabilities.min_image_extent.width, capabilities.max_image_extent.width),
                height: height.clamp(capabilities.min_image_extent.height, capabilities.max_image_extent.height),
            }
        }
    }
    
    fn create_image_views(
        device: &VulkanDevice,
        images: &[vk::Image],
        format: vk::Format,
    ) -> Result<Vec<vk::ImageView>> {
        images
            .iter()
            .map(|&image| {
                let create_info = vk::ImageViewCreateInfo {
                    image,
                    view_type: vk::ImageViewType::TYPE_2D,
                    format,
                    components: vk::ComponentMapping::default(),
                    subresource_range: vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                    ..Default::default()
                };
                
                unsafe { device.handle().create_image_view(&create_info, None) }
                    .map_err(CompositorError::from)
            })
            .collect()
    }
    
    /// Acquire the next image for rendering
    pub fn acquire_next_image(&mut self) -> Result<u32> {
        // Simplified - in real implementation would use semaphores
        let (image_index, _) = unsafe {
            self.swapchain_loader.acquire_next_image(
                self.swapchain,
                u64::MAX,
                vk::Semaphore::null(),
                vk::Fence::null(),
            )?
        };
        
        self.current_image = image_index;
        Ok(image_index)
    }
    
    /// Present the current image
    pub fn present(&self) -> Result<()> {
        // Simplified - in real implementation would use proper queue submission
        let swapchains = [self.swapchain];
        let image_indices = [self.current_image];
        
        let _present_info = vk::PresentInfoKHR {
            swapchain_count: 1,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: image_indices.as_ptr(),
            ..Default::default()
        };
        
        // Note: This should use the present queue from device
        // For now, using a simplified approach
        
        Ok(())
    }
    
    /// Get current extent
    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }
    
    /// Get swapchain images
    pub fn images(&self) -> &Vec<vk::Image> {
        &self.images
    }
    
    /// Get swapchain image views  
    pub fn image_views(&self) -> &Vec<vk::ImageView> {
        &self.image_views
    }
    
    /// Get swapchain format
    pub fn format(&self) -> vk::Format {
        self.format
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        // Note: In a real implementation, device should be passed for cleanup
        info!("Swapchain destroyed");
    }
}
