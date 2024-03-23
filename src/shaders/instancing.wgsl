struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,

    @location(3) i_pos_scale: vec4<f32>,
    @location(4) i_color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let position = vertex.position * vertex.i_pos_scale.w + vertex.i_pos_scale.xyz;
    var out: VertexOutput;
    out.color = vertex.i_color;
    out.clip_position = (vec4<f32>(position, 1.0) - camera.position) * camera.view_proj * quadMatrix;
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct CameraUniform {
    view_proj: mat4x4f,
    position: vec4<f32>,
}

const vertex1: vec4<f32> = vec4<f32>(-1.0, -1.0, 0.0, 1.0); // bottom left corner
const vertex2: vec4<f32> = vec4<f32>(1.0, -1.0, 0.0, 1.0); // bottom right corner
const vertex3: vec4<f32> = vec4<f32>(1.0, 1.0, 0.0, 1.0); // top right corner
const vertex4: vec4<f32> = vec4<f32>(-1.0, 1.0, 0.0, 1.0); // top left corner

// Combine vertices into a quad matrix
const quadMatrix: mat4x4f = mat4x4(
  vec4f(vertex1.x, vertex1.y, vertex1.z, vertex1.w),
  vec4f(vertex2.x, vertex2.y, vertex2.z, vertex2.w),
  vec4f(vertex3.x, vertex3.y, vertex3.z, vertex3.w),
  vec4f(vertex4.x, vertex4.y, vertex4.z, vertex4.w)
);