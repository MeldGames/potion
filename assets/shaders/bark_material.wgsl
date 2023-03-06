#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings
#import bevy_pbr::mesh_functions

#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::shadows

#import bevy_shader_utils::perlin_noise_3d

// TODO Hook up own utils.wgsl and load_internal
fn rgb2hsv(color: vec3<f32>) -> vec3<f32>{
    let K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    let P = mix(vec4(color.bg, K.wz), vec4(color.gb, K.xy), step(color.b, color.g));
    let Q = mix(vec4(P.xyw, color.r), vec4(color.r, P.yzx), step(P.x, color.r));

    let d = Q.x - min(Q.w, Q.y);
    let e = 1.0e-10;
    return vec3(abs(Q.z + (Q.w - Q.y) / (6.0 * d + e)), d / (Q.x + e), Q.x);

}

struct BarkMaterial {
    color: vec4<f32>,
};


struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
#ifdef VERTEX_UVS
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
    @location(3) tangent: vec4<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(4) color: vec4<f32>,
#endif
#ifdef SKINNED
    @location(5) joint_indices: vec4<u32>,
    @location(6) joint_weights: vec4<f32>,
#endif
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
#ifdef VERTEX_UVS
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
    @location(3) world_tangent: vec4<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(4) color: vec4<f32>,
#endif
    @location(5) normal : vec3<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {

    let winddir = normalize(vec3(0.5, 1.0, 0.0));
    let windspeed = 0.9;
    let windstrength = 0.1;
    let z = winddir * windspeed * globals.time;

    let a = vec2(vertex.position.x, vertex.position.z);
    var noise = perlinNoise3(vertex.position + z);
    
    
    var out: VertexOutput;
    out.clip_position = mesh_position_world_to_clip(out.world_position);
    var bend = 1.0;
#ifdef VERTEX_COLORS
    out.color = out.clip_position;
    bend = vertex.color.g;
#endif
    let position = (  noise * windstrength * bend) + vertex.position;
#ifdef SKINNED
    var model = skin_model(vertex.joint_indices, vertex.joint_weights);
    out.world_normal = skin_normals(model, vertex.normal);
#else
    var model = mesh.model;
    out.world_normal = mesh_normal_local_to_world(vertex.normal);
#endif
    // out.world_position = mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));
    out.world_position = mesh_position_local_to_world(model, vec4<f32>(position, 1.0));
#ifdef VERTEX_UVS
    out.uv = vertex.uv;
#endif
#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_tangent_local_to_world(model, vertex.tangent);
#endif


    out.normal = vertex.normal;

    return out;
}

@group(1) @binding(0)
var<uniform> material: BarkMaterial;
@group(1) @binding(1)
var base_color_texture: texture_2d<f32>;
@group(1) @binding(2)
var base_color_sampler: sampler;
@group(1) @binding(3)
var alpha_texture: texture_2d<f32>;
@group(1) @binding(4)
var alpha_sampler: sampler;

struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
    @location(5) normal : vec3<f32>,
};


@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {    

    // albedo
    var output_color: vec4<f32> = material.color;
    

    // mask and output
        return vec4(output_color);

    //return vec4(under3 , 1.0);
    //return in.world_position * vec4(in.world_normal, 1.0);
}
