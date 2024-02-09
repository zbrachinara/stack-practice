use bevy::math::uvec2;
use bevy::prelude::*;
use image::{GenericImage, ImageBuffer};
use tap::Tap;

/// Assuming that each texture is equal in size, this function combines them into a single texture
/// which can be bound as a `texture_2d_array`. If this assumption doesn't pass, the function
/// panics. It also panics if there are no images to stack.
pub fn stack_images(images: &[Handle<Image>], server: &Assets<Image>) -> Image {
    // fetch an image to determine the target size
    let size = server.get(&images[0]).unwrap().size();
    let buffer_size = size * uvec2(1, images.len() as u32);
    // create the buffer from the inferred size
    let mut buffer = ImageBuffer::new(buffer_size.x, buffer_size.y);

    // copy each image into the newly created buffer
    for (i, h) in images.iter().enumerate() {
        let image = server.get(h).unwrap();
        let dyn_image = image.clone().try_into_dynamic().unwrap();
        buffer
            .copy_from(&dyn_image, 0, size.y * i as u32)
            .expect("Failed to copy image while creating an image stack");
    }

    Image::from_dynamic(buffer.into(), true).tap_mut(|i| {
        i.reinterpret_stacked_2d_as_array(images.len() as u32);
    })
}
