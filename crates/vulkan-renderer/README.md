# Vulkan Rendering Engine

This specialized crate provides a high-performance Vulkan-based rendering pipeline optimized for compositor operations and ultra-high-resolution display environments.

## Shader Pipeline Architecture

The rendering engine employs an automated shader compilation system integrated with the build process, ensuring optimal SPIR-V bytecode generation and deployment:

- **Build-time compilation**: GLSL shaders are automatically transpiled to SPIR-V during compilation
- **Runtime optimization**: Optional dynamic shader compilation capabilities via the `shaderc` crate
- **Dependency requirements**: 
  - `glslc` compiler from the Vulkan SDK must be accessible in system PATH
  - Alternative: Runtime compilation through integrated `shaderc` dependency

## Build Configuration

```bash
# Standard build with integrated shader compilation
cargo build

# Clean rebuild forcing complete shader recompilation
cargo clean && cargo build
```

## Shader Architecture

- **`surface.vert`** - Optimized vertex shader for surface geometry processing and transformation
- **`surface.frag`** - Advanced fragment shader implementing texture sampling with filtering optimization

The build system automatically generates SPIR-V bytecode artifacts in `target/shaders/` and embeds them as compile-time resources for optimal runtime performance.
