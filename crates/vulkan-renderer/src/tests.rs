// Tests for vulkan-renderer 4K capabilities
// These tests validate our claims about 4K graphics performance and memory management

use crate::{VulkanInstance, VulkanDevice};
use ash::vk;

// Test configuration for 4K displays
const TEST_4K_WIDTH: u32 = 3840;
const TEST_4K_HEIGHT: u32 = 2160;
const TEST_4K_FRAMEBUFFER_SIZE: u64 = (TEST_4K_WIDTH * TEST_4K_HEIGHT * 4) as u64; // RGBA8

/// Test helper to create a basic Vulkan instance for testing
fn create_test_instance() -> Result<VulkanInstance, Box<dyn std::error::Error>> {
    let app_info = vk::ApplicationInfo::builder()
        .application_name(c"compositor_test")
        .application_version(vk::make_api_version(0, 1, 0, 0))
        .engine_name(c"custom_compositor_test")
        .engine_version(vk::make_api_version(0, 1, 0, 0))
        .api_version(vk::API_VERSION_1_3)
        .build();

    VulkanInstance::new_with_info(&app_info, &[])
        .map_err(|e| format!("Failed to create test instance: {:?}", e).into())
}

/// Test helper to create a basic Vulkan device for testing
fn create_test_device(instance: &VulkanInstance) -> Result<VulkanDevice, Box<dyn std::error::Error>> {
    let physical_devices = instance.enumerate_physical_devices()
        .map_err(|e| format!("Failed to enumerate physical devices: {:?}", e))?;
    
    if physical_devices.is_empty() {
        return Err("No Vulkan physical devices found".into());
    }

    let physical_device = physical_devices[0];
    VulkanDevice::new_with_device(instance, physical_device, &[], &[])
        .map_err(|e| format!("Failed to create test device: {:?}", e).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_4k_swapchain_creation() {
        // Test that we can create swapchains suitable for 4K displays
        let instance = match create_test_instance() {
            Ok(instance) => instance,
            Err(e) => {
                eprintln!("Skipping test - no Vulkan support: {}", e);
                return;
            }
        };

        let device = match create_test_device(&instance) {
            Ok(device) => device,
            Err(e) => {
                eprintln!("Skipping test - no suitable device: {}", e);
                return;
            }
        };

        // Check if the device supports the required surface formats for 4K
        let surface_formats = [
            vk::Format::B8G8R8A8_SRGB,
            vk::Format::R8G8B8A8_SRGB,
            vk::Format::B8G8R8A8_UNORM,
            vk::Format::R8G8B8A8_UNORM,
        ];

        let format_properties = surface_formats.iter().map(|&format| {
            instance.get_physical_device_format_properties(device.physical_device(), format)
        }).collect::<Vec<_>>();

        // Verify at least one format supports optimal tiling and color attachment
        let has_suitable_format = format_properties.iter().any(|props| {
            props.optimal_tiling_features.contains(
                vk::FormatFeatureFlags::COLOR_ATTACHMENT | vk::FormatFeatureFlags::BLIT_DST
            )
        });

        assert!(has_suitable_format, "No suitable surface format found for 4K rendering");

        // Check memory requirements for 4K framebuffers
        let memory_properties = instance.get_physical_device_memory_properties(device.physical_device());

        let total_device_memory: u64 = memory_properties.memory_heaps
            .iter()
            .take(memory_properties.memory_heap_count as usize)
            .filter(|heap| heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL))
            .map(|heap| heap.size)
            .sum();

        // We need at least 32MB for a 4K framebuffer with double buffering
        let required_memory = TEST_4K_FRAMEBUFFER_SIZE * 2;
        assert!(
            total_device_memory >= required_memory,
            "Insufficient GPU memory for 4K rendering. Required: {}MB, Available: {}MB",
            required_memory / (1024 * 1024),
            total_device_memory / (1024 * 1024)
        );
    }

    #[test]
    fn test_4k_memory_allocation() {
        let instance = match create_test_instance() {
            Ok(instance) => instance,
            Err(e) => {
                eprintln!("Skipping test - no Vulkan support: {}", e);
                return;
            }
        };

        let device = match create_test_device(&instance) {
            Ok(device) => device,
            Err(e) => {
                eprintln!("Skipping test - no suitable device: {}", e);
                return;
            }
        };

        // Test image creation for 4K resolution
        let image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk::Format::B8G8R8A8_SRGB)
            .extent(vk::Extent3D {
                width: TEST_4K_WIDTH,
                height: TEST_4K_HEIGHT,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let test_image = unsafe {
            device.handle().create_image(&image_create_info, None)
        };

        match test_image {
            Ok(image) => {
                // Get memory requirements for the 4K image
                let memory_requirements = unsafe {
                    device.handle().get_image_memory_requirements(image)
                };

                // Verify the memory size is reasonable for 4K
                let expected_min_size = TEST_4K_FRAMEBUFFER_SIZE;
                assert!(
                    memory_requirements.size >= expected_min_size,
                    "Memory requirements too small for 4K image: {} bytes, expected at least {} bytes",
                    memory_requirements.size,
                    expected_min_size
                );

                // Clean up
                unsafe {
                    device.handle().destroy_image(image, None);
                }
            }
            Err(e) => {
                panic!("Failed to create 4K test image: {:?}", e);
            }
        }
    }

    #[test]
    fn test_gpu_capabilities() {
        let instance = match create_test_instance() {
            Ok(instance) => instance,
            Err(e) => {
                eprintln!("Skipping test - no Vulkan support: {}", e);
                return;
            }
        };

        let device = match create_test_device(&instance) {
            Ok(device) => device,
            Err(e) => {
                eprintln!("Skipping test - no suitable device: {}", e);
                return;
            }
        };

        // Check device properties for 4K capability
        let device_properties = instance.get_physical_device_properties(device.physical_device());

        // Verify maximum image dimensions support 4K+
        let limits = device_properties.limits;
        assert!(
            limits.max_image_dimension2_d >= TEST_4K_WIDTH.max(TEST_4K_HEIGHT),
            "GPU doesn't support 4K image dimensions. Max: {}, Required: {}",
            limits.max_image_dimension2_d,
            TEST_4K_WIDTH.max(TEST_4K_HEIGHT)
        );

        // Check for required features
        let device_features = instance.get_physical_device_features(device.physical_device());

        assert!(
            device_features.sampler_anisotropy == vk::TRUE,
            "GPU doesn't support anisotropic filtering required for high-quality 4K rendering"
        );

        // Verify sufficient memory bandwidth (heuristic check)
        let memory_properties = instance.get_physical_device_memory_properties(device.physical_device());

        let device_local_memory: u64 = memory_properties.memory_heaps
            .iter()
            .take(memory_properties.memory_heap_count as usize)
            .filter(|heap| heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL))
            .map(|heap| heap.size)
            .sum();

        // Minimum 1GB for comfortable 4K rendering
        const MIN_VRAM_4K: u64 = 1024 * 1024 * 1024;
        assert!(
            device_local_memory >= MIN_VRAM_4K,
            "Insufficient VRAM for 4K rendering. Available: {}MB, Recommended: {}MB",
            device_local_memory / (1024 * 1024),
            MIN_VRAM_4K / (1024 * 1024)
        );
    }

    #[test]
    fn test_multi_surface_rendering() {
        let instance = match create_test_instance() {
            Ok(instance) => instance,
            Err(e) => {
                eprintln!("Skipping test - no Vulkan support: {}", e);
                return;
            }
        };

        let device = match create_test_device(&instance) {
            Ok(device) => device,
            Err(e) => {
                eprintln!("Skipping test - no suitable device: {}", e);
                return;
            }
        };

        // Test creating multiple render targets for multi-surface scenario
        let surface_count = 4; // Simulate multiple windows/surfaces
        let mut test_images = Vec::new();

        for i in 0..surface_count {
            let image_create_info = vk::ImageCreateInfo::builder()
                .image_type(vk::ImageType::TYPE_2D)
                .format(vk::Format::B8G8R8A8_SRGB)
                .extent(vk::Extent3D {
                    width: 1920, // HD per surface for multi-surface test
                    height: 1080,
                    depth: 1,
                })
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let image = unsafe {
                device.handle().create_image(&image_create_info, None)
            }.expect(&format!("Failed to create test surface {}", i));

            test_images.push(image);
        }

        // Verify all surfaces were created successfully
        assert_eq!(test_images.len(), surface_count, "Failed to create all test surfaces");

        // Calculate total memory footprint
        let total_memory: u64 = test_images.iter().map(|&image| {
            let memory_req = unsafe { device.handle().get_image_memory_requirements(image) };
            memory_req.size
        }).sum();

        // Verify memory usage is reasonable for multi-surface rendering
        let memory_properties = instance.get_physical_device_memory_properties(device.physical_device());

        let available_memory: u64 = memory_properties.memory_heaps
            .iter()
            .take(memory_properties.memory_heap_count as usize)
            .filter(|heap| heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL))
            .map(|heap| heap.size)
            .sum();

        // Should use less than 25% of available memory for this test
        assert!(
            total_memory < available_memory / 4,
            "Multi-surface memory usage too high: {}MB of {}MB available",
            total_memory / (1024 * 1024),
            available_memory / (1024 * 1024)
        );

        // Clean up
        for image in test_images {
            unsafe {
                device.handle().destroy_image(image, None);
            }
        }
    }

    #[test]
    fn test_performance_baseline() {
        use std::time::Instant;

        let instance = match create_test_instance() {
            Ok(instance) => instance,
            Err(e) => {
                eprintln!("Skipping test - no Vulkan support: {}", e);
                return;
            }
        };

        let device = match create_test_device(&instance) {
            Ok(device) => device,
            Err(e) => {
                eprintln!("Skipping test - no suitable device: {}", e);
                return;
            }
        };

        // Performance test: measure command buffer creation and submission time
        let cmd_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(0); // Assuming queue family 0 for this test

        let command_pool = unsafe {
            device.handle().create_command_pool(&cmd_pool_create_info, None)
        }.expect("Failed to create command pool for performance test");

        let cmd_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffers = unsafe {
            device.handle().allocate_command_buffers(&cmd_buffer_allocate_info)
        }.expect("Failed to allocate command buffer");

        let cmd_buffer = command_buffers[0];

        // Measure command buffer begin/end time
        let start = Instant::now();

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            device.handle().begin_command_buffer(cmd_buffer, &begin_info)
                .expect("Failed to begin command buffer");
            device.handle().end_command_buffer(cmd_buffer)
                .expect("Failed to end command buffer");
        }

        let duration = start.elapsed();

        // Command buffer operations should be fast (within reasonable bounds)
        // Note: First-time operations may include driver setup overhead
        assert!(
            duration.as_millis() < 5,
            "Command buffer creation too slow: {}ms (should be < 5ms)",
            duration.as_millis()
        );

        // Clean up
        unsafe {
            device.handle().destroy_command_pool(command_pool, None);
        }

        // Test memory allocation performance
        let start = Instant::now();

        let buffer_create_info = vk::BufferCreateInfo::builder()
            .size(1024 * 1024) // 1MB buffer
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let test_buffer = unsafe {
            device.handle().create_buffer(&buffer_create_info, None)
        }.expect("Failed to create test buffer");

        let allocation_time = start.elapsed();

        // Buffer creation should be fast
        assert!(
            allocation_time.as_millis() < 10,
            "Buffer allocation too slow: {}ms (should be < 10ms)",
            allocation_time.as_millis()
        );

        unsafe {
            device.handle().destroy_buffer(test_buffer, None);
        }
    }
}
