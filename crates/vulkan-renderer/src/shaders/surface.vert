#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 texCoord;

layout(location = 0) out vec2 fragTexCoord;

layout(push_constant) uniform PushConstants {
    mat4 transform;
    vec2 offset;
    vec2 scale;
} pushConstants;

void main() {
    vec2 pos = position * pushConstants.scale + pushConstants.offset;
    gl_Position = pushConstants.transform * vec4(pos, 0.0, 1.0);
    fragTexCoord = texCoord;
}
