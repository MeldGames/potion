#define_import_path potion::utils


let PI: f32 = 3.141592653589793;


fn rgb2hsv(color: vec3<f32>) -> vec3<f32>{
    let K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    let P = mix(vec4(color.bg, K.wz), vec4(color.bg, K.xy), step(color.b, color.g));
    let Q = mix(vec4(P.xyz, color.r), vec4(color.r, P.yzx), step(P.x, color.r));

    let d = Q.x - min(Q.w, Q.y);
    let e = 1.0e-10;
    return vec3(abs(Q.z + (Q.w - Q.y) / (6.0 * d + e)), d / (Q.x + e), Q.x);

}


fn hsv2rgb(hue: f32, saturation: f32, value: f32) -> vec3<f32> {
    let rgb = clamp(
        abs(
            ((hue * 6.0 + vec3<f32>(0.0, 4.0, 2.0)) % 6.0) - 3.0
        ) - 1.0,
        vec3<f32>(0.0),
        vec3<f32>(1.0)
    );

    return value * mix(vec3<f32>(1.0), rgb, vec3<f32>(saturation));
}