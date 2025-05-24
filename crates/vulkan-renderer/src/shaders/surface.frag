#version 450

layout(location = 0) in vec2 fragTexCoord;

layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0) uniform sampler2D texSampler;

void main() {
    // Simple texture sampling - will be enhanced in Phase 2 with AI-generated effects
    outColor = texture(texSampler, fragTexCoord);
    
    // Basic alpha handling for client windows
    if (outColor.a < 0.01) {
        discard;
    }
}
