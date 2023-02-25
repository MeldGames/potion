
#import bevy_pbr::mesh_view_bindings
#import bevy_shader_utils::perlin_noise_2d

struct LeafMaterial {
    color: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> material: LeafMaterial;
@group(1) @binding(1)
var base_color_texture: texture_2d<f32>;
@group(1) @binding(2)
var base_color_sampler: sampler;
@group(1) @binding(3)
var alpha_texture: texture_2d<f32>;
@group(1) @binding(4)
var alpha_sampler: sampler;


@fragment
fn fragment(
    @builtin(front_facing) is_front: bool,
    @builtin(position) coord: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    #ifdef VERTEX_COLORS
    @location(4) color: vec4<f32>,
    #endif
) -> @location(0) vec4<f32> {
    // return material.color * textureSample(base_color_texture, base_color_sampler, uv);
    // var input: vec3<f32> = vec3<f32>(uv.x * 40.0, uv.y * 40.0, 1.);
    //var noise = simplexNoise3(vec3<f32>(world_normal.xyz * 10.0));
    //var alpha = (noise + 2.0) / 2.0;

    var cutout = textureSample(alpha_texture, alpha_sampler, uv);
    let minus = vec3(1.0,-8.0,0.0);

    // Colors
    let green1 = vec4<f32>(0.7, 0.95, 0.25, 1.0);
    let green2 = vec4<f32>(0.11, 0.6, 0.3, 1.0);

    let world_pos_norm = normalize(world_position.xyz);
    let norm = world_position * vec4(world_normal, 1.0);

    let mask = (world_pos_norm.x + world_pos_norm.y + world_pos_norm.z) /3.0;




    // Noise
    // var input1: vec2<f32> = vec2<f32>(world_position.x /5.0, globals.time);
    // var input2: vec2<f32> = vec2<f32>(world_position.y /5.0, globals.time);
    // var input3: vec2<f32> = vec2<f32>(world_position.z /5.0, globals.time);

    // var noise1 = perlinNoise2(input1);
    // var noise2 = perlinNoise2(input2);
    // var noise3 = perlinNoise2(input3);

    // var value1 = (noise1 + 1.0) /2.0;
    // var value2 = (noise2 + 1.0) /2.0;
    // var value3 = (noise3 + 1.0) /2.0;



    //return textureSample(base_color_texture, base_color_sampler, uv) * vec4<f32>(1.0, 1.0, 1.0, alpha);
    //return vec4<f32>(0.0, 0.6, 0.2, cutout.a * alpha);

    //return  (world_position + vec4(minus, 1.0)) /4.0 ;

    //return world_position * vec4(world_normal, 1.0);

    //return  vec4(normalize(view.world_position.xyz - world_position.xyz), 1.0);
    //return  (normalize(world_position) + vec4(0.2,-0.8,0.2, 1.0)) /0.1 ;
    //return vec4<f32>(normalize(vec3(value1, value2, value3)), 1.0);
    let color = mix(green2, green1, mask);

    //return vec4(dot(normalize(vec3(world_position.x, world_position.y, world_position.z)), normalize(vec3(1.0,1.0,1.0)) )) ;
    //return vec4(mask);

    return vec4(color * cutout.a);
    //var t = sin(uv.x * 6.28 *1.0 + -0.5) *0.5 + 0.5;
    //t = abs(fract(uv.x * 5.0) * 2.0 - 1.0);
    //return vec4(t);

    //return cutout;
    // return vec4<f32>(uv.x, uv.y, 0.0, 1.0);
    //return vec4<f32>(normal, 1.0);

  
}
