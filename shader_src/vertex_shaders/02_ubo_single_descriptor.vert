#version 460

struct CameraData {
    vec4 position;
    float aspect_ratio;
};

struct ObjectData {
    vec4 position;
    float scale;
};

layout(std140, set = 0, binding = 0) uniform GlobalBuffer {
    CameraData camera_data;
} global_buffer;

layout(std140, set = 0, binding = 1) readonly buffer ObjectBuffer {
    ObjectData object_data[];
} object_buffer;

layout(push_constant) uniform ResourceIndices {
    uint object_index;
} resource_indices;

layout(location = 0) in vec3 position;

void main()
{
    vec4 object_position = object_buffer.object_data[resource_indices.object_index].position;
    float object_scale = object_buffer.object_data[resource_indices.object_index].scale;

    // We only use x,y from the vertex input, but treat it as a 3D point extended to 4D
    vec4 world_position = object_position + vec4(position * object_scale, 0.0);

    vec4 camera_position = global_buffer.camera_data.position;
    float camera_aspect_ratio = global_buffer.camera_data.aspect_ratio;

    vec4 view_position = (world_position - camera_position);

    // Apply aspect ratio correction to X
    view_position.x /= camera_aspect_ratio;

    gl_Position = vec4(view_position.xyz, 1.0);
}
