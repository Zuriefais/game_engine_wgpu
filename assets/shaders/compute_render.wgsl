@group(0) @binding(0)
var out_texture: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(16 , 16)
fn main(@builtin(global_invocation_id) global_id: vec3u) {
   textureStore(out_texture, vec2<u32>(global_id.x, global_id.y), vec4<f32>(1.0, 0.0, 0.0, 1.0));
}
