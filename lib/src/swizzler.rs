use std::ops::{Index, IndexMut};

use image::{
    DynamicImage,
    RgbaImage,
    Rgba,
    GenericImageView
};
use crate::errors::{
    ErrorKind
};

type SwizzleResult<T> = Result<T, ErrorKind>;

pub struct ChannelDescriptor<'a> {
    pub channel: u8,
    pub img: &'a DynamicImage,
}

pub struct FlatSamplesDescriptor<'a> {
    channel: u8,
    flat_samples: image::FlatSamples<&'a [ u8 ]>
}

impl FlatSamplesDescriptor<'_> {

    fn get_sample(&self, x: u32, y: u32) -> Option<&u8> {
        self.flat_samples.get_sample(self.channel, x, y)
    }

}

impl<'a> From<ChannelDescriptor<'a>> for FlatSamplesDescriptor<'a> {

    fn from(e: ChannelDescriptor<'a>) -> FlatSamplesDescriptor<'a> {
        FlatSamplesDescriptor {
            channel: e.channel,
            flat_samples: e.img.as_flat_samples()
        }
    }

}

impl<'a> ChannelDescriptor<'a> {

    pub fn new(
        img: &'a DynamicImage,
        channel: u8
    ) -> ChannelDescriptor {
        ChannelDescriptor { img, channel }
    }

}

pub trait Swizzle<'a, T> {

    fn swizzle(channels: &T) -> Result<DynamicImage, ErrorKind>;

}

fn retrieve_dimensions<'a>(samples: &[ Option<FlatSamplesDescriptor<'a>> ]) -> Result<(u32, u32), ErrorKind>
{
    let mut dimensions: Option<(u32, u32)> = None;

    let array = samples.as_ref();
    for optional in array {
        if let Some(sample) = optional {
            let img_dim = (sample.flat_samples.layout.width, sample.flat_samples.layout.height);
            dimensions.get_or_insert(img_dim);
            if img_dim != dimensions.unwrap() {
                return Err(ErrorKind::InvalidSize);
            }
        }
    }

    dimensions.ok_or(ErrorKind::InvalidSize)
}

fn swizzle_generic<'a, Pixel>(
    samples: &[ Option<FlatSamplesDescriptor<'a>> ]
) -> SwizzleResult<image::ImageBuffer<Pixel, Vec<Pixel::Subpixel>>>
where
    Pixel: 'static + image::Pixel<Subpixel = u8> + Sized + Index<usize>
{
    let dimensions = retrieve_dimensions(samples)?;
    let mut image: image::ImageBuffer<Pixel, Vec<Pixel::Subpixel>> = image::ImageBuffer::new(
        dimensions.0,
        dimensions.1
    );
    let pixels = image.enumerate_pixels_mut();

    let nb_samples = samples.len();

    for (x, y, pixel) in pixels {
        let channels = pixel.channels_mut();
        for i in 0..nb_samples {
            if let Some(sample) = &samples[i] {
                channels[i] = *sample.get_sample(x, y).unwrap();
            }
        }
    }

    Ok(image)
}

macro_rules! test_macro {
    ( $p:expr, $( $x:ident ),* ) => {
        {
            let samples = [
                $(
                    $x.map(|channel| FlatSamplesDescriptor::from(channel)),
                )*
            ];

            let dimensions = retrieve_dimensions(&samples)?;
            let mut image = image::ImageBuffer::from_pixel(
                dimensions.0,
                dimensions.1,
                $p
            );
            let pixels = image.enumerate_pixels_mut();
            let nb_samples = samples.len();

            for (x, y, pixel) in pixels {
                for i in 0..nb_samples {
                    if let Some(sample) = &samples[i] {
                        pixel[i] = *sample.get_sample(x, y).unwrap();
                    }
                }
            }

            Ok(image)
        }
    };
}

pub fn swizzle_rgb(
    r: Option<ChannelDescriptor>,
    g: Option<ChannelDescriptor>,
    b: Option<ChannelDescriptor>
) -> SwizzleResult<image::RgbImage> {
    swizzle_generic(&[
        r.map(|channel| FlatSamplesDescriptor::from(channel)),
        g.map(|channel| FlatSamplesDescriptor::from(channel)),
        b.map(|channel| FlatSamplesDescriptor::from(channel))
    ])
}

pub fn swizzle_rgba(
    r: Option<ChannelDescriptor>,
    g: Option<ChannelDescriptor>,
    b: Option<ChannelDescriptor>,
    a: Option<ChannelDescriptor>,
) -> SwizzleResult<image::RgbaImage> {
    test_macro!(Rgba([ 0, 0, 0, 255 ]), r, g, b, a)
}
