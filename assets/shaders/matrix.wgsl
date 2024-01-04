#import bevy_pbr::forward_io::VertexOutput

@group(1) @binding(0) var<uniform> dimensions: vec2u;
@group(1) @binding(1) var mino_textures: texture_2d_array<f32>;
@group(1) @binding(2) var mino_textures_sampler: sampler;
@group(1) @binding(3) var<storage, read> data: array<u32>;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4f {
    let cell_size = 1f / vec2<f32>(dimensions);

    let cell_position = in.uv / cell_size;
    let integral_position_flipped = vec2<u32>(floor(cell_position));
    let integral_position =
        vec2u(integral_position_flipped.x, dimensions.y - integral_position_flipped.y - 1u);
    let ix = integral_position.y * dimensions.x + integral_position.x;
    let cell_type = data[ix];

    let cell_inner_position = cell_position - floor(cell_position);

    return textureSample(mino_textures, mino_textures_sampler, cell_inner_position, cell_type);
}