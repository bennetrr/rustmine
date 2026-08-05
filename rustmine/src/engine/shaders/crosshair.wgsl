struct Uniforms {
    resolution: vec2<f32>,
}
@group(0) @binding(0) var<uniform> u: Uniforms;

@vertex
fn vs_main(@location(0) pixel_pos: vec2<f32>) -> @builtin(position) vec4<f32> {
    let ndc = (pixel_pos / u.resolution) * 2.0 - 1.0;
    return vec4<f32>(ndc.x, -ndc.y, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0); // white
}