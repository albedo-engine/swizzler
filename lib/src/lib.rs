use std::collections::HashMap;

use image::{
    ColorType,
    DynamicImage,
    Pixel,
    RgbImage
};

pub mod errors;

mod swizzler;
pub use swizzler::{
    ChannelDescriptor,
    Swizzle,
    swizzle_rgb,
    swizzle_rgba,
};

/* trait Swizzle {

    fn swizzle(&self) -> DynamicImage;

}

pub fn swizzle_rgb(
    dimensions: (u32, u32),
    r: &ChannelDescriptor,
    g: &ChannelDescriptor,
    b: &ChannelDescriptor
) -> RgbImage {

    let mut image = RgbImage::new(dimensions.0, dimensions.1);
    let pixels = image.enumerate_pixels_mut();

    let r_flat = r.img.as_flat_samples();
    let g_flat = g.img.as_flat_samples();
    let b_flat = b.img.as_flat_samples();

    for (x, y, pixel) in pixels {
        (*pixel)[0] = *r_flat.get_sample(r.channel, x, y).unwrap();
        (*pixel)[1] = *g_flat.get_sample(g.channel, x, y).unwrap();
        (*pixel)[2] = *b_flat.get_sample(b.channel, x, y).unwrap();
    }

    image
}

pub fn swizzle_rgba(
    dimensions: (u32, u32),
    r: &ChannelDescriptor,
    g: &ChannelDescriptor,
    b: &ChannelDescriptor,
    a: Option<&ChannelDescriptor>
) -> RgbImage {

    let mut image = RgbImage::new(dimensions.0, dimensions.1);
    let pixels = image.enumerate_pixels_mut();

    let r_flat = r.img.as_flat_samples();
    let g_flat = g.img.as_flat_samples();
    let b_flat = b.img.as_flat_samples();

    for (x, y, pixel) in pixels {
        (*pixel)[0] = *r_flat.get_sample(r.channel, x, y).unwrap();
        (*pixel)[1] = *g_flat.get_sample(g.channel, x, y).unwrap();
        (*pixel)[2] = *b_flat.get_sample(b.channel, x, y).unwrap();
    }

    image
} */
