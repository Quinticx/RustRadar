#import bevy_pbr::mesh_functions::{get_model_matrix, mesh_position_local_to_clip}

struct InstanceUniforms {
    alpha_power: f32,
};
//@group(3) @binding(0) var<uniform> instance_uniforms: InstanceUniforms;
// @group(0) @binding(100) var<uniform> instance_uniforms: InstanceUniforms;



struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,

    //@location(3) i_pos_scale: vec4<f32>,
    @location(4) i_color: vec4<f32>,
    @location(5) i_alpha: f32,
    @location(6) i_t_x: vec4<f32>,
    @location(7) i_t_y: vec4<f32>,
    @location(8) i_t_z: vec4<f32>,
    @location(9) i_t_w: vec4<f32>,
    //@location(5) i_rotation: array<f32, 16>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let transform = mat4x4f(vertex.i_t_x, vertex.i_t_y, vertex.i_t_z, vertex.i_t_w);
    let position = vertex.position;// * vertex.i_pos_scale.w + vertex.i_pos_scale.xyz;
    var out: VertexOutput;

    out.clip_position = mesh_position_local_to_clip(
        transform * get_model_matrix(0u),
        vec4<f32>(vertex.position, 1.0)
    );
    out.color = vec4(vertex.i_color.rgb, pow(vertex.i_color.a, vertex.i_alpha));
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}