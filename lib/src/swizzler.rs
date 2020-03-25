use crate::errors::ErrorKind;
use image::{DynamicImage, Luma, LumaA, Rgb, Rgba};

use paste;

type SwizzleResult<T> = Result<T, ErrorKind>;
type SwizzleResultDyn = Result<DynamicImage, ErrorKind>;
type ChannelDescResult = Result<ChannelDescriptor, ErrorKind>;

pub struct ChannelDescriptor {
    pub channel: u8,
    pub img: std::rc::Rc<image::DynamicImage>,
}

impl Clone for ChannelDescriptor {
    fn clone(&self) -> Self {
        ChannelDescriptor {
            channel: self.channel,
            img: self.img.clone(),
        }
    }
}

impl ChannelDescriptor {
    pub fn from_image_rc(
        img_input: &std::rc::Rc<image::DynamicImage>,
        channel: u8,
    ) -> ChannelDescResult {
        let img = std::rc::Rc::clone(img_input);
        Ok(ChannelDescriptor { img: img, channel })
    }

    pub fn from_image(img_input: image::DynamicImage, channel: u8) -> ChannelDescResult {
        let img = std::rc::Rc::new(img_input);
        Ok(ChannelDescriptor { img, channel })
    }

    pub fn from_path<T>(path: T, channel: u8) -> ChannelDescResult
    where
        T: AsRef<std::path::Path>,
    {
        let img = image::open(path.as_ref())?;
        ChannelDescriptor::from_image(img, channel)
    }

    pub fn from_description<T>(input: T) -> ChannelDescResult
    where
        T: AsRef<str>,
    {
        let mut split = input.as_ref().split(':');

        let img_path = String::from(split.next().ok_or(ErrorKind::InvalidDescriptorString(
            String::from(input.as_ref()),
        ))?);

        let last = split
            .next()
            .ok_or(ErrorKind::InvalidDescriptorString(String::from(
                input.as_ref(),
            )))?;

        let channel = (last.parse::<u8>().map_err(|_e| -> ErrorKind {
            ErrorKind::InvalidDescriptorString(String::from("failed to parse channel"))
        }))?;
        let img = image::open(&img_path)?;

        ChannelDescriptor::from_image(img, channel)
    }
}

macro_rules! swizzle {
    ( $p:expr, $( $x:ident ),* ) => {
        {
            paste::expr! {
                let mut dimensions: Option<(u32, u32)> = None;

                $(
                    let [<$x _channel>]: u8 = match $x {
                        Some(desc) => desc.channel,
                        None => 0
                    };

                    let [<flat_ $x>] = match $x {
                        Some(desc) => Some(desc.img.as_ref().as_flat_samples()),
                        None => None
                    };

                    if let Some(sample) = &[<flat_ $x>] {
                        let img_dim = (
                            sample.layout.width,
                            sample.layout.height
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

                // TODO: change `i` to recursive macro computing index.
                let mut i = 0;
                let pixels = image.enumerate_pixels_mut();

                for (x, y, pixel) in pixels {
                    $(
                        if let Some(sample) = &[<flat_ $x>] {
                            pixel[i] = *sample.get_sample(
                                [<$x _channel>], x, y
                            ).unwrap();
                        }
                        i += 1;
                    )*
                    i = 0;
                }

                Ok(image)
            }
        }
    };
}

pub fn to_luma(r: &Option<ChannelDescriptor>) -> SwizzleResult<image::GrayImage> {
    static DEFAULT: Luma<u8> = Luma([0]);
    swizzle!(DEFAULT, r)
}

pub fn to_luma_dyn(r: &Option<ChannelDescriptor>) -> SwizzleResultDyn {
    Ok(DynamicImage::ImageLuma8(to_luma(r)?))
}

#[allow(non_snake_case)]
pub fn to_lumaA(
    r: &Option<ChannelDescriptor>,
    a: &Option<ChannelDescriptor>,
) -> SwizzleResult<image::GrayAlphaImage> {
    static DEFAULT: LumaA<u8> = LumaA([0, 255]);
    swizzle!(DEFAULT, r, a)
}

pub fn to_lumaa_dyn(
    r: &Option<ChannelDescriptor>,
    g: &Option<ChannelDescriptor>,
) -> SwizzleResultDyn {
    Ok(DynamicImage::ImageLumaA8(to_lumaA(r, g)?))
}

pub fn to_rgb(
    r: &Option<ChannelDescriptor>,
    g: &Option<ChannelDescriptor>,
    b: &Option<ChannelDescriptor>,
) -> SwizzleResult<image::RgbImage> {
    static DEFAULT: Rgb<u8> = Rgb([0, 0, 0]);
    swizzle!(DEFAULT, r, g, b)
}

pub fn to_rgb_dyn(
    r: &Option<ChannelDescriptor>,
    g: &Option<ChannelDescriptor>,
    b: &Option<ChannelDescriptor>,
) -> SwizzleResultDyn {
    Ok(DynamicImage::ImageRgb8(to_rgb(r, g, b)?))
}

pub fn to_rgba(
    r: &Option<ChannelDescriptor>,
    g: &Option<ChannelDescriptor>,
    b: &Option<ChannelDescriptor>,
    a: &Option<ChannelDescriptor>,
) -> SwizzleResult<image::RgbaImage> {
    static DEFAULT: Rgba<u8> = Rgba([0, 0, 0, 255]);
    swizzle!(DEFAULT, r, g, b, a)
}

pub fn to_rgba_dyn(
    r: &Option<ChannelDescriptor>,
    g: &Option<ChannelDescriptor>,
    b: &Option<ChannelDescriptor>,
    a: &Option<ChannelDescriptor>,
) -> SwizzleResultDyn {
    Ok(DynamicImage::ImageRgba8(to_rgba(r, g, b, a)?))
}

pub fn to_dynamic(
    descriptors: &Vec<Option<ChannelDescriptor>>,
) -> SwizzleResult<image::DynamicImage> {
    let dynimg = match descriptors.len() {
        1 => image::DynamicImage::ImageLuma8(to_luma(&descriptors[0])?),
        2 => image::DynamicImage::ImageLumaA8(to_lumaA(&descriptors[0], &descriptors[1])?),
        3 => image::DynamicImage::ImageRgb8(to_rgb(
            &descriptors[0],
            &descriptors[1],
            &descriptors[2],
        )?),
        a if a >= 4 => image::DynamicImage::ImageRgba8(to_rgba(
            &descriptors[0],
            &descriptors[1],
            &descriptors[2],
            &descriptors[3],
        )?),
        _ => panic!("too big vector!"),
    };
    Ok(dynimg)
}
