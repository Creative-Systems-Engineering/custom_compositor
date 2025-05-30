use ash::{vk, Entry, Instance};
use compositor_utils::prelude::*;
use std::ffi::{CStr, CString};

/// Vulkan instance wrapper with validation layers for development
#[derive(Clone)]
pub struct VulkanInstance {
    entry: Entry,
    instance: Instance,
    debug_utils: Option<DebugUtils>,
    api_version: u32,
}

struct DebugUtils {
    loader: ash::extensions::ext::DebugUtils,
    messenger: vk::DebugUtilsMessengerEXT,
}

// Manual Clone implementation since DebugUtils doesn't derive Clone
impl Clone for DebugUtils {
    fn clone(&self) -> Self {
        Self {
            loader: self.loader.clone(),
            messenger: self.messenger,
        }
    }
}

impl VulkanInstance {
    /// Create a new Vulkan instance with default parameters
    pub fn new() -> Result<Self> {
        // Application info
        let app_name = CString::new("Custom Compositor")?;
        let engine_name = CString::new("Custom Engine")?;
        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(vk::make_api_version(0, 0, 1, 0))
            .engine_name(&engine_name)
            .engine_version(vk::make_api_version(0, 0, 1, 0))
            .api_version(vk::API_VERSION_1_3)
            .build();
        
        Self::new_with_info(&app_info, &[])
    }
    
    /// Create a new Vulkan instance with custom application info and extensions
    /// 
    /// This flexible constructor allows specifying custom application information and additional
    /// instance extensions beyond the default set required for compositor operation.
    /// 
    /// # Arguments
    /// * `app_info` - Custom application information including name, version, and API requirements
    /// * `extensions` - Additional instance extensions to enable beyond compositor defaults
    /// 
    /// # Default Extensions Included
    /// * Surface extension for window management
    /// * Platform-specific surface extensions (Xlib, Wayland)
    /// * Debug utilities (in debug builds)
    /// 
    /// # Returns
    /// A configured VulkanInstance ready for device creation and graphics operations.
    /// 
    /// # Examples
    /// ```rust
    /// let app_info = vk::ApplicationInfo::builder()
    ///     .application_name(c"my_compositor")
    ///     .api_version(vk::API_VERSION_1_3)
    ///     .build();
    /// let instance = VulkanInstance::new_with_info(&app_info, &[])?;
    /// ```
    pub fn new_with_info(app_info: &vk::ApplicationInfo, extensions: &[*const i8]) -> Result<Self> {
        let entry = Entry::linked();
        
        // Check API version
        let api_version = entry
            .try_enumerate_instance_version()?
            .unwrap_or(vk::make_api_version(0, 1, 0, 0));
        
        info!("Vulkan API version: {}", format_version(api_version));
        
        // Required extensions
        let mut extension_names = vec![
            ash::extensions::khr::Surface::name().as_ptr(),
            ash::extensions::khr::XlibSurface::name().as_ptr(),
            ash::extensions::khr::WaylandSurface::name().as_ptr(),
        ];
        
        // Add provided extensions
        extension_names.extend_from_slice(extensions);
        
        // Add debug extensions in debug mode
        let debug_enabled = cfg!(debug_assertions);
        if debug_enabled {
            extension_names.push(ash::extensions::ext::DebugUtils::name().as_ptr());
        }
        
        // Validation layers
        let layer_names = if debug_enabled {
            vec![CString::new("VK_LAYER_KHRONOS_validation")?]
        } else {
            vec![]
        };
        let layer_names_raw: Vec<*const i8> = layer_names
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();
        
        // Create instance
        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(app_info)
            .enabled_layer_names(&layer_names_raw)
            .enabled_extension_names(&extension_names)
            .build();
        
        let instance = unsafe { entry.create_instance(&create_info, None)? };
        
        // Setup debug messenger
        let debug_utils = if debug_enabled {
            Some(Self::setup_debug_messenger(&entry, &instance)?)
        } else {
            None
        };
        
        info!("Vulkan instance created successfully");
        
        Ok(Self {
            entry,
            instance,
            debug_utils,
            api_version,
        })
    }
    
    /// Get a reference to the raw ash Entry
    /// 
    /// Provides access to the Vulkan entry point for low-level operations.
    /// Used by test suites and advanced graphics operations that need direct Vulkan access.
    pub fn entry(&self) -> &Entry {
        &self.entry
    }
    
    /// Get a reference to the raw ash Instance
    /// 
    /// Provides access to the Vulkan instance handle for operations requiring direct instance access.
    /// Essential for device enumeration, surface creation, and other instance-level operations.
    pub fn handle(&self) -> &Instance {
        &self.instance
    }
    
    /// Get the supported Vulkan API version
    /// 
    /// Returns the highest Vulkan API version supported by the system.
    /// Used for feature detection and compatibility checking.
    pub fn api_version(&self) -> u32 {
        self.api_version
    }
    
    /// Enumerate all available physical devices (GPUs)
    /// 
    /// Returns a list of all Vulkan-capable physical devices in the system.
    /// Essential for GPU selection and capability validation in multi-GPU systems.
    /// Used by test suites to validate 4K graphics capabilities.
    pub fn enumerate_physical_devices(&self) -> Result<Vec<vk::PhysicalDevice>> {
        let devices = unsafe { self.instance.enumerate_physical_devices() }
            .map_err(|e| CompositorError::graphics(format!("Failed to enumerate physical devices: {:?}", e)))?;
        Ok(devices)
    }
    
    /// Get comprehensive properties for a physical device
    /// 
    /// Returns detailed device properties including limits, features, and capabilities.
    /// Used for GPU selection, capability validation, and performance optimization.
    /// Critical for 4K graphics validation and memory allocation planning.
    pub fn get_physical_device_properties(&self, device: vk::PhysicalDevice) -> vk::PhysicalDeviceProperties {
        unsafe { self.instance.get_physical_device_properties(device) }
    }
    
    /// Get available features for a physical device
    /// 
    /// Returns the set of optional features supported by the device.
    /// Used for feature detection and enabling advanced graphics capabilities.
    pub fn get_physical_device_features(&self, device: vk::PhysicalDevice) -> vk::PhysicalDeviceFeatures {
        unsafe { self.instance.get_physical_device_features(device) }
    }
    
    /// Get memory properties for a physical device
    /// 
    /// Returns detailed memory heap and type information for optimal memory allocation.
    /// Critical for 4K framebuffer allocation and performance optimization.
    /// Used by test suites to validate memory requirements for high-resolution rendering.
    pub fn get_physical_device_memory_properties(&self, device: vk::PhysicalDevice) -> vk::PhysicalDeviceMemoryProperties {
        unsafe { self.instance.get_physical_device_memory_properties(device) }
    }
    
    /// Get format properties for a specific image format on a device
    /// 
    /// Returns supported operations, tiling modes, and features for image formats.
    /// Essential for surface format selection and render target compatibility.
    pub fn get_physical_device_format_properties(&self, device: vk::PhysicalDevice, format: vk::Format) -> vk::FormatProperties {
        unsafe { self.instance.get_physical_device_format_properties(device, format) }
    }
    
    fn setup_debug_messenger(entry: &Entry, instance: &Instance) -> Result<DebugUtils> {
        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT {
            message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE,
            message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            pfn_user_callback: Some(vulkan_debug_callback),
            ..Default::default()
        };
        
        let debug_utils_loader = ash::extensions::ext::DebugUtils::new(entry, instance);
        let debug_callback = unsafe {
            debug_utils_loader.create_debug_utils_messenger(&debug_info, None)?
        };
        
        Ok(DebugUtils {
            loader: debug_utils_loader,
            messenger: debug_callback,
        })
    }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        unsafe {
            if let Some(ref debug_utils) = self.debug_utils {
                debug_utils.loader.destroy_debug_utils_messenger(debug_utils.messenger, None);
            }
            self.instance.destroy_instance(None);
        }
        info!("Vulkan instance destroyed");
    }
}

// Debug callback for Vulkan validation layers
unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    _message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number = callback_data.message_id_number;
    
    let message_id_name = if callback_data.p_message_id_name.is_null() {
        std::borrow::Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };
    
    let message = if callback_data.p_message.is_null() {
        std::borrow::Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };
    
    match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
            error!("[Vulkan] {} ({}): {}", message_id_name, message_id_number, message);
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            warn!("[Vulkan] {} ({}): {}", message_id_name, message_id_number, message);
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
            info!("[Vulkan] {} ({}): {}", message_id_name, message_id_number, message);
        }
        _ => {
            debug!("[Vulkan] {} ({}): {}", message_id_name, message_id_number, message);
        }
    }
    
    vk::FALSE
}

fn format_version(version: u32) -> String {
    format!(
        "{}.{}.{}",
        vk::api_version_major(version),
        vk::api_version_minor(version),
        vk::api_version_patch(version)
    )
}
