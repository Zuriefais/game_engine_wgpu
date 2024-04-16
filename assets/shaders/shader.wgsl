struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color_index: u32,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color_index = instance.color;
    out.clip_position = (vec4<f32>(model.position+vec3f(instance.position, 0.0), 1.0) - camera.position) * camera.view_proj;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return pallete.values[in.color_index];
}

struct CameraUniform {
    view_proj: mat4x4f,
    position: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct Pallete {
  values: array<vec4<f32>, 16>,
};

@group(0) @binding(1) var<uniform> pallete: Pallete;

struct InstanceInput {
    @location(5) position: vec2<f32>,
    @location(7) color: u32,
};
