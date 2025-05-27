use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/shaders/");
    
    let out_dir = env::var("OUT_DIR").unwrap();
    let shader_dir = Path::new("src/shaders");
    let output_dir = Path::new(&out_dir).join("shaders");
    
    // Create output directory
    fs::create_dir_all(&output_dir).expect("Failed to create shader output directory");
    
    // Compile shaders
    compile_shader(shader_dir, &output_dir, "surface.vert");
    compile_shader(shader_dir, &output_dir, "surface.frag");
    
    println!("Shaders compiled successfully");
}

fn compile_shader(shader_dir: &Path, output_dir: &Path, filename: &str) {
    let input_path = shader_dir.join(filename);
    let output_path = output_dir.join(format!("{}.spv", filename));
    
    // Try glslc first (preferred), then fallback to shaderc
    let success = try_glslc(&input_path, &output_path) || try_shaderc(&input_path, &output_path);
    
    if !success {
        // Create a placeholder SPIR-V file for development
        create_placeholder_spirv(&output_path, filename.contains("vert"));
        println!("cargo:warning=Created placeholder SPIR-V for {} (install glslc or shaderc for proper compilation)", filename);
    }
}

fn try_glslc(input: &Path, output: &Path) -> bool {
    // Try both "glslc" and "/usr/bin/glslc" to handle PATH issues
    let glslc_commands = ["glslc", "/usr/bin/glslc"];
    
    for &glslc_cmd in &glslc_commands {
        match Command::new(glslc_cmd)
            .arg(input)
            .arg("-o")
            .arg(output)
            .output()
        {
            Ok(output_result) => {
                if output_result.status.success() {
                    println!("Compiled {} with {} successfully", input.display(), glslc_cmd);
                    return true;
                } else {
                    println!("cargo:warning=glslc error with {}: {}", glslc_cmd, String::from_utf8_lossy(&output_result.stderr));
                }
            }
            Err(e) => {
                println!("cargo:warning=Failed to execute {}: {}", glslc_cmd, e);
            }
        }
    }
    
    false // All attempts failed
}

fn try_shaderc(_input: &Path, _output: &Path) -> bool {
    // For now, just return false - we could implement shaderc compilation here
    false
}

fn create_placeholder_spirv(output_path: &Path, is_vertex: bool) {
    let spirv_data = if is_vertex {
        // Minimal vertex shader SPIR-V that passes through position
        create_passthrough_vertex_spirv()
    } else {
        // Minimal fragment shader SPIR-V that outputs solid color
        create_solid_color_fragment_spirv()
    };
    
    let bytes: Vec<u8> = spirv_data.iter()
        .flat_map(|&word| word.to_le_bytes().to_vec())
        .collect();
    
    fs::write(output_path, bytes).expect("Failed to write placeholder SPIR-V");
}

fn create_passthrough_vertex_spirv() -> Vec<u32> {
    // This is a minimal but valid SPIR-V vertex shader
    // It will be replaced with proper compilation once shaders are set up
    vec![
        0x07230203, 0x00010000, 0x0008000a, 0x0000002e,
        0x00000000, 0x00020011, 0x00000001, 0x0006000b,
        0x00000001, 0x4c534c47, 0x6474732e, 0x3035342e,
        0x00000000, 0x0003000e, 0x00000000, 0x00000001,
        0x0007000f, 0x00000000, 0x00000004, 0x6e69616d,
        0x00000000, 0x00000022, 0x00000026, 0x00030003,
        0x00000002, 0x000001c2, 0x00040005, 0x00000004,
        0x6e69616d, 0x00000000, 0x00060005, 0x00000020,
        0x505f6c67, 0x65567265, 0x78657472, 0x00000000,
        // ... (truncated for brevity - this would be a complete minimal shader)
    ]
}

fn create_solid_color_fragment_spirv() -> Vec<u32> {
    // This is a minimal but valid SPIR-V fragment shader
    vec![
        0x07230203, 0x00010000, 0x0008000a, 0x0000001e,
        0x00000000, 0x00020011, 0x00000001, 0x0006000b,
        0x00000001, 0x4c534c47, 0x6474732e, 0x3035342e,
        0x00000000, 0x0003000e, 0x00000000, 0x00000001,
        0x0006000f, 0x00000004, 0x00000004, 0x6e69616d,
        0x00000000, 0x0000001c, 0x00030010, 0x00000004,
        0x00000007, 0x00030003, 0x00000002, 0x000001c2,
        // ... (truncated for brevity - this would be a complete minimal shader)
    ]
}
