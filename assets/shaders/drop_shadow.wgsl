#import bevy_pbr::forward_io::VertexOutput;

@group(1) @binding(1) var texture: texture_1d<f32>;
@group(1) @binding(2) var samp: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(texture, samp, in.uv.x);
    let color_transparent = vec4f(color.rgb, 0.0);
    let g = min(in.uv.y / 0.1, 1.0);

    let rapidity = 10000.0;
    let s = log(g * (rapidity - 1.0) + 1.0) / log(rapidity);

    return mix(color, color_transparent, s);
}