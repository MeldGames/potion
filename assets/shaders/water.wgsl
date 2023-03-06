
#import bevy_pbr::mesh_view_bindings
#import bevy_shader_utils::perlin_noise_2d
#import bevy_pbr::utils

struct WaterMaterial {
    color: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> material: WaterMaterial;
@group(1) @binding(1)
var base_color_texture: texture_2d<f32>;
@group(1) @binding(2)
var base_color_sampler: sampler;


@fragment
fn fragment(
    @builtin(front_facing) is_front: bool,
    @builtin(position) coord: vec4<f32>,
    @builtin(sample_index) sample_index: u32,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    #ifdef VERTEX_COLORS
    @location(4) color: vec4<f32>,
    #endif
) -> @location(0) vec4<f32> {

    let depth = prepass_depth(coord, sample_index);
    return vec4(depth, depth, depth, 1.0);


  

  
}
