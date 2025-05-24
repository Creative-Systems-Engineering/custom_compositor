# Vulkan Renderer Crate

This crate handles all Vulkan rendering operations for the compositor.

## Shader Compilation

Shaders are automatically compiled during build. Make sure you have:
- `glslc` (from Vulkan SDK) in your PATH
- Or `shaderc` crate for runtime compilation

## Building

```bash
# Build with shader compilation
cargo build

# Force shader recompilation
cargo clean && cargo build
```

## Shaders

- `surface.vert` - Vertex shader for surface quads
- `surface.frag` - Fragment shader for texture sampling

Generated SPIR-V files are placed in `target/shaders/` and embedded at build time.
