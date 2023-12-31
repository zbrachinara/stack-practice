use bevy::{
    math::{IVec2, UVec2},
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::Image,
    },
};

use crate::board::CELL_SIZE;

/// An image that has no
pub fn transparent_texture(size: UVec2) -> Image {
    Image::new_fill(
        Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
    )
}

/// This function FLIPS the image of `src` in the y direction, and it also flips `location` in the y
/// direction relative to standard bevy coordinates (that is, y points down).
///
/// Copies data from `src` into a region in `dst`. The region is described by `location`. It is
/// interpreted as a square with length `CELL_SIZE`, positioned at the given coordinate *after*
/// scaled by `CELL_SIZE`.
///
/// Essentially each image is treated as a grid, and one grid square is copied from src to dst.
pub(crate) fn copy_from_to(dst: &mut Image, src: &Image, location: IVec2) {
    let width = dst.width();
    let location = location.as_uvec2() * CELL_SIZE;
    let region = (location.y..location.y + CELL_SIZE).map(|col| {
        let offset = ((location.x + col * width) * 4) as usize;
        let offset_end = offset + (CELL_SIZE as usize) * 4;
        (offset, offset_end)
    });

    src.data
        .chunks_exact(CELL_SIZE as usize * 4)
        .zip(region)
        .for_each(|(src, (a, b))| {
            dst.data[a..b].copy_from_slice(src);
        })
}
