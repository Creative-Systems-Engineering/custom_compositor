use ash::{vk, Device};
use compositor_utils::prelude::*;
use crate::instance::VulkanInstance;
use std::ffi::CStr;

/// Vulkan logical device wrapper
#[derive(Clone)]
pub struct VulkanDevice {
    physical_device: vk::PhysicalDevice,
    device: Device,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    #[allow(dead_code)] // Will be used for queue submission and synchronization
    graphics_queue_family: u32,
    #[allow(dead_code)] // Will be used for presentation and queue management
    present_queue_family: u32,
    device_properties: vk::PhysicalDeviceProperties,
}

impl VulkanDevice {
    /// Create a new Vulkan device with automatic physical device selection
    pub fn new(instance: &VulkanInstance) -> Result<Self> {
        let physical_devices = instance.enumerate_physical_devices()?;
        
        if physical_devices.is_empty() {
            return Err(CompositorError::init("No Vulkan-capable devices found"));
        }
        
        // Select the best physical device
        let (physical_device, _graphics_queue_family, _present_queue_family) = 
            Self::select_physical_device(instance, &physical_devices)?;
        
        Self::new_with_device(instance, physical_device, &[], &[])
    }
    
    /// Create a new Vulkan device with specific physical device and configuration options
    /// 
    /// This flexible constructor allows creating a logical device from a pre-selected physical device
    /// with custom extensions and features. Essential for test suites and advanced configurations
    /// where automatic device selection is not appropriate.
    /// 
    /// # Arguments
    /// * `instance` - The VulkanInstance containing the physical device
    /// * `physical_device` - Pre-selected physical device (GPU) to create the logical device from
    /// * `_extensions` - Device extensions to enable (currently unused, reserved for future features)
    /// * `_features` - Device features to enable (currently unused, reserved for future features)
    /// 
    /// # Device Creation Process
    /// 1. Finds appropriate graphics and present queue families
    /// 2. Logs selected GPU information for debugging
    /// 3. Creates logical device with required queues
    /// 4. Retrieves queue handles for immediate use
    /// 
    /// # Returns
    /// A configured VulkanDevice ready for rendering operations, memory allocation, and command submission.
    /// 
    /// # Used By
    /// * 4K graphics validation test suite
    /// * Custom GPU selection scenarios
    /// * Advanced compositor configurations
    pub fn new_with_device(
        instance: &VulkanInstance, 
        physical_device: vk::PhysicalDevice,
        _extensions: &[*const i8],
        _features: &[vk::PhysicalDeviceFeatures] // Placeholder for future feature selection
    ) -> Result<Self> {
        // Find queue families
        let (graphics_queue_family, present_queue_family) = 
            Self::find_queue_families(instance, physical_device)?;
        
        // Get device properties
        let device_properties = unsafe {
            instance.handle().get_physical_device_properties(physical_device)
        };
        
        info!("Selected GPU: {}", unsafe {
            CStr::from_ptr(device_properties.device_name.as_ptr())
                .to_string_lossy()
        });
        
        // Create logical device
        let device = Self::create_logical_device(
            instance, 
            physical_device, 
            graphics_queue_family, 
            present_queue_family
        )?;
        
        // Get queue handles
        let graphics_queue = unsafe { device.get_device_queue(graphics_queue_family, 0) };
        let present_queue = unsafe { device.get_device_queue(present_queue_family, 0) };
        
        Ok(Self {
            physical_device,
            device,
            graphics_queue,
            present_queue,
            graphics_queue_family,
            present_queue_family,
            device_properties,
        })
    }
    
    /// Find appropriate graphics and present queue families for a physical device
    /// 
    /// Searches through the available queue families to find ones capable of graphics operations
    /// and presentation. Essential for compositor operation as these queues handle all rendering
    /// and display output operations.
    /// 
    /// # Arguments
    /// * `instance` - VulkanInstance containing the physical device
    /// * `physical_device` - Physical device to query for queue families
    /// 
    /// # Queue Family Selection Logic
    /// 1. Enumerates all available queue families on the device
    /// 2. Finds the first queue family with graphics capability
    /// 3. Currently uses the same family for present (simplified for testing)
    /// 4. In production, would verify surface presentation support
    /// 
    /// # Returns
    /// A tuple containing (graphics_queue_family_index, present_queue_family_index)
    /// 
    /// # Error Conditions
    /// Returns error if no graphics-capable queue family is found, indicating the device
    /// cannot perform the rendering operations required by the compositor.
    fn find_queue_families(
        instance: &VulkanInstance, 
        physical_device: vk::PhysicalDevice
    ) -> Result<(u32, u32)> {
        let queue_families = unsafe {
            instance.handle().get_physical_device_queue_family_properties(physical_device)
        };
        
        let graphics_family = queue_families
            .iter()
            .enumerate()
            .find(|(_, family)| family.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .map(|(index, _)| index as u32);
        
        if let Some(graphics_family) = graphics_family {
            // For now, use the same queue family for present
            // In a real implementation, we'd check surface support
            let present_family = graphics_family;
            Ok((graphics_family, present_family))
        } else {
            Err(CompositorError::init("No suitable queue family found"))
        }
    }
    
    fn select_physical_device(
        instance: &VulkanInstance,
        devices: &[vk::PhysicalDevice],
    ) -> Result<(vk::PhysicalDevice, u32, u32)> {
        for &device in devices {
            let properties = unsafe {
                instance.handle().get_physical_device_properties(device)
            };
            
            // Prefer discrete GPUs for 4K performance
            let is_discrete = properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU;
            
            // Find graphics queue family
            let queue_families = unsafe {
                instance.handle().get_physical_device_queue_family_properties(device)
            };
            
            let graphics_family = queue_families
                .iter()
                .enumerate()
                .find(|(_, family)| family.queue_flags.contains(vk::QueueFlags::GRAPHICS))
                .map(|(index, _)| index as u32);
            
            if let Some(graphics_family) = graphics_family {
                // For now, use the same queue family for present
                // In a real implementation, we'd check surface support
                let present_family = graphics_family;
                
                info!("Found suitable device: {} ({})", 
                      unsafe { CStr::from_ptr(properties.device_name.as_ptr()).to_string_lossy() },
                      if is_discrete { "discrete" } else { "integrated" });
                
                return Ok((device, graphics_family, present_family));
            }
        }
        
        Err(CompositorError::init("No suitable graphics device found"))
    }
    
    fn create_logical_device(
        instance: &VulkanInstance,
        physical_device: vk::PhysicalDevice,
        graphics_queue_family: u32,
        present_queue_family: u32,
    ) -> Result<Device> {
        let queue_priorities = [1.0f32];
        
        // Create unique queue families
        let mut unique_families = std::collections::HashSet::new();
        unique_families.insert(graphics_queue_family);
        unique_families.insert(present_queue_family);
        
        let queue_create_infos: Vec<_> = unique_families
            .into_iter()
            .map(|family| {
                vk::DeviceQueueCreateInfo {
                    queue_family_index: family,
                    queue_count: queue_priorities.len() as u32,
                    p_queue_priorities: queue_priorities.as_ptr(),
                    ..Default::default()
                }
            })
            .collect();
        
        // Required device extensions
        let device_extensions = [
            ash::extensions::khr::Swapchain::name().as_ptr(),
        ];
        
        // Device features
        let device_features = vk::PhysicalDeviceFeatures::default();
        
        let device_create_info = vk::DeviceCreateInfo {
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr(),
            enabled_extension_count: device_extensions.len() as u32,
            pp_enabled_extension_names: device_extensions.as_ptr(),
            p_enabled_features: &device_features,
            ..Default::default()
        };
        
        let device = unsafe {
            instance.handle().create_device(physical_device, &device_create_info, None)?
        };
        
        Ok(device)
    }
    
    /// Get reference to the logical device handle
    /// 
    /// Provides access to the Vulkan logical device for direct GPU operations.
    /// Used by rendering operations, memory allocation, and resource creation.
    /// Essential for test suites and advanced graphics operations.
    pub fn handle(&self) -> &Device {
        &self.device
    }
    
    /// Get the physical device handle
    /// 
    /// Returns the underlying physical device (GPU) that this logical device represents.
    /// Used for capability queries, memory property inspection, and device selection logic.
    /// Critical for validation tests and hardware capability detection.
    pub fn physical_device(&self) -> vk::PhysicalDevice {
        self.physical_device
    }
    
    /// Get the graphics queue handle
    /// 
    /// Returns the graphics queue used for rendering commands and GPU operations.
    /// All rendering operations, command buffer submissions, and graphics workloads
    /// are submitted through this queue for execution on the GPU.
    pub fn graphics_queue(&self) -> vk::Queue {
        self.graphics_queue
    }
    
    /// Get the presentation queue handle
    /// 
    /// Returns the queue used for presenting rendered frames to the display.
    /// Essential for swapchain operations and getting rendered content to the screen.
    /// In many cases this is the same as the graphics queue for efficiency.
    pub fn present_queue(&self) -> vk::Queue {
        self.present_queue
    }
    
    /// Get human-readable device name for debugging and user information
    /// 
    /// Returns the GPU's marketing name as reported by the driver.
    /// Useful for logging, debugging, and providing user feedback about
    /// the selected graphics hardware.
    /// 
    /// # Examples
    /// * "NVIDIA GeForce RTX 4080"
    /// * "AMD Radeon RX 7800 XT"  
    /// * "Intel Arc A770"
    pub fn get_device_name(&self) -> String {
        unsafe {
            CStr::from_ptr(self.device_properties.device_name.as_ptr())
                .to_string_lossy()
                .into_owned()
        }
    }
    
    /// Get the vendor ID for hardware identification
    /// 
    /// Returns the PCI vendor ID for the graphics hardware.
    /// Used for vendor-specific optimizations and compatibility checks.
    /// 
    /// # Common Vendor IDs
    /// * 0x10DE - NVIDIA
    /// * 0x1002 - AMD
    /// * 0x8086 - Intel
    pub fn get_vendor_id(&self) -> u32 {
        self.device_properties.vendor_id
    }
    
    /// Get complete device properties for capability inspection
    /// 
    /// Returns the full set of device properties including limits, features,
    /// and hardware specifications. Used for detailed capability analysis
    /// and performance optimization decisions.
    pub fn properties(&self) -> &vk::PhysicalDeviceProperties {
        &self.device_properties
    }
    
    /// Wait for all GPU operations to complete
    /// 
    /// Blocks until the GPU has finished all pending operations on this device.
    /// Critical for safe resource cleanup and ensuring proper synchronization
    /// before application shutdown or major state changes.
    /// 
    /// # Warning
    /// This is a synchronizing operation that can cause performance hitches.
    /// Should only be used during cleanup or when explicit synchronization is required.
    pub fn wait_idle(&self) -> Result<()> {
        unsafe {
            self.device.device_wait_idle()?;
        }
        Ok(())
    }

    /// Get human-readable device type description
    /// 
    /// Returns a descriptive string indicating the type of graphics hardware.
    /// Useful for performance expectations and capability assumptions.
    /// 
    /// # Device Type Classifications
    /// * **Discrete GPU**: Dedicated graphics card with own memory
    /// * **Integrated GPU**: CPU-integrated graphics sharing system memory
    /// * **Virtual GPU**: Virtualized graphics in VM environments
    /// * **CPU**: Software-only rendering fallback
    pub fn get_device_type(&self) -> String {
        match self.device_properties.device_type {
            vk::PhysicalDeviceType::DISCRETE_GPU => "Discrete GPU".to_string(),
            vk::PhysicalDeviceType::INTEGRATED_GPU => "Integrated GPU".to_string(),
            vk::PhysicalDeviceType::VIRTUAL_GPU => "Virtual GPU".to_string(),
            vk::PhysicalDeviceType::CPU => "CPU".to_string(),
            _ => "Other".to_string(),
        }
    }
}

impl Drop for VulkanDevice {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
        }
        info!("Vulkan device destroyed");
    }
}
