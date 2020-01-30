use std::ops::{Index, IndexMut};

use image::{
    Luma,
    LumaA,
    Rgb,
    Rgba
};
use crate::errors::{
    ErrorKind
};

type SwizzleResult<T> = Result<T, ErrorKind>;

pub struct ChannelDescriptor<'a> {
    pub channel: u8,
    pub img: &'a image::DynamicImage,
}

impl<'a> ChannelDescriptor<'a> {

    pub fn new(
        img: &'a image::DynamicImage,
        channel: u8
    ) -> ChannelDescriptor {
        ChannelDescriptor { img, channel }
    }

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

macro_rules! swizzle {
    ( $p:expr, $( $x:ident ),* ) => {
        {
            let mut dimensions: Option<(u32, u32)> = None;
            $(
                let $x = $x.map(|channel| FlatSamplesDescriptor::from(channel));

                if let Some(sample) = &$x {
                    let img_dim = (
                        sample.flat_samples.layout.width,
                        sample.flat_samples.layout.height
                    );
                    dimensions.get_or_insert(img_dim);
                    if img_dim != dimensions.unwrap() {
                        return Err(ErrorKind::InvalidSize);
                    }
                }
            )*

            let dimensions = dimensions.ok_or(ErrorKind::InvalidSize)?;
            let mut image = image::ImageBuffer::from_pixel(
                dimensions.0,
                dimensions.1,
                $p
            );
            let mut i = 0;
            let pixels = image.enumerate_pixels_mut();

            for (x, y, pixel) in pixels {
                $(
                    if let Some(sample) = &$x {
                        pixel[i] = *sample.get_sample(x, y).unwrap();
                    }
                    i += 1;
                )*
                i = 0;
            }

            Ok(image)
        }
    };
}

pub fn to_luma(
    r: Option<ChannelDescriptor>
) -> SwizzleResult<image::GrayImage> {
    static DEFAULT: Luma<u8> = Luma([ 0 ]);
    swizzle!(DEFAULT, r)
}

pub fn to_lumaA(
    r: Option<ChannelDescriptor>,
    a: Option<ChannelDescriptor>
) -> SwizzleResult<image::GrayAlphaImage> {
    static DEFAULT: LumaA<u8> = LumaA([ 0, 255 ]);
    swizzle!(DEFAULT, r, a)
}

pub fn to_rgb(
    r: Option<ChannelDescriptor>,
    g: Option<ChannelDescriptor>,
    b: Option<ChannelDescriptor>
) -> SwizzleResult<image::RgbImage> {
    static DEFAULT: Rgb<u8> = Rgb([ 0, 0, 0 ]);
    swizzle!(DEFAULT, r, g, b)
}

pub fn to_rgba(
    r: Option<ChannelDescriptor>,
    g: Option<ChannelDescriptor>,
    b: Option<ChannelDescriptor>,
    a: Option<ChannelDescriptor>,
) -> SwizzleResult<image::RgbaImage> {
    static DEFAULT: Rgba<u8> = Rgba([ 0, 0, 0, 255 ]);
    swizzle!(DEFAULT, r, g, b, a)
}
