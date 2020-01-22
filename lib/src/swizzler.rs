/*use image::{
    ColorType
};

pub struct SwizzlerBuilder {



}

pub trait CommandBuilder<Channels> {

    new(c: Channels, type: image::ColorType)

}

pub struct Command<Channels> {

    image_type: image::ColorType,

    channels: Channels

}*/

use image::{
    DynamicImage,
    GenericImageView
};
use crate::errors::{
    ErrorKind
};

pub struct ChannelDescriptor<'a> {
    pub channel: u8,
    pub img: &'a DynamicImage,
}

impl<'a> ChannelDescriptor<'a> {

    pub fn new(
        img: &'a DynamicImage,
        channel: u8
    ) -> ChannelDescriptor {
        ChannelDescriptor { img, channel }
    }

}

pub trait Swizzle<Channels> {

    fn swizzle(channels: Channels) -> Result<DynamicImage, ErrorKind>;

}

impl Swizzle<[ Option<ChannelDescriptor<'_>>; 4 ]> for image::RgbaImage {

    fn swizzle(channels: [ Option<ChannelDescriptor>; 4 ]) ->
        Result<DynamicImage, ErrorKind> {

        let mut dimensions: Option<(u32, u32)> = None;

        for optional in &channels {
            if let Some(channel) = optional {
                let img_dim = channel.img.dimensions();
                dimensions.get_or_insert(img_dim);
                if img_dim != dimensions.unwrap() {
                    return Err(ErrorKind::InvalidSize);
                }
            }
        }

        let dimensions = match dimensions {
            Some(val) => val,
            _ => return Err(ErrorKind::InvalidSize)
        };

        let mut image = image::RgbaImage::new(dimensions.0, dimensions.1);
        let pixels = image.enumerate_pixels_mut();

        let r_flat = r.img.as_flat_samples();
        let g_flat = g.img.as_flat_samples();
        let b_flat = b.img.as_flat_samples();

        for (x, y, pixel) in pixels {
            (*pixel)[0] = *r_flat.get_sample(r.channel, x, y).unwrap();
            (*pixel)[1] = *g_flat.get_sample(g.channel, x, y).unwrap();
            (*pixel)[2] = *b_flat.get_sample(b.channel, x, y).unwrap();
        }

        Ok(DynamicImage::ImageRgba8(image))
    }

}
