use crate::errors::ErrorKind;
use image::{DynamicImage, Luma, LumaA, Rgb, Rgba};

use paste;

type SwizzleResult<T> = Result<T, ErrorKind>;
type SwizzleResultDyn = Result<DynamicImage, ErrorKind>;
type ChannelDescResult = Result<ChannelDescriptor, ErrorKind>;

/// Input source descriptor.
///
/// This type provides an pointer to an allocated image, as well as a channel
/// index, representing the source to read from the image.
///
/// Channel descriptors are used to feed swizzling. They describe how to extract
/// source pixel data.
///
/// **NOTE**: careful when creating descriptor, the channel used **must** exists.
/// For instance, trying to access the second channel of a _Grayscale_ image
/// will raise an error.
///
/// # Examples
///
/// You can create a [`ChannelDescriptor`] by using a path and the channel index:
///
/// ```
/// use std::path::PathBuf;
/// // Creates a descriptor pointing to the **red** channel of the `input.png` image.
/// let descriptor = ChannelDescriptor::from_path(PathBuf::from("./input.png", 0);
/// ```
///
/// You can also create a descriptor by using an image already in memory:
///
/// ```
/// // Creates a descriptor pointing to the **red** channel of the image `my_image`.
/// let descriptor = ChannelDescriptor::from_image(my_image, 0);
/// ```
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

/// Generates a _Grayscale_ image from a single descriptor
///
/// # Arguments
///
/// * `r` - The descriptor to use for writing the _red_ channel
///
/// # Example
///
/// ```
/// let descriptor = ChannelDescriptor::from_path(PathBuf::from("./input.png"), 0);
/// let image = to_luma(&descriptor);
/// ```
pub fn to_luma(r: &ChannelDescriptor) -> SwizzleResult<image::GrayImage> {
    static DEFAULT: Luma<u8> = Luma([0]);
    let descriptor = Some(r);
    swizzle!(DEFAULT, descriptor)
}

/// Generates a _Grayscale_ image from a single descriptor, and wraps it into
/// a `image::DynamicImage`
///
/// # Arguments
///
/// * `r` - The descriptor to use for writing the _red_ channel
pub fn to_luma_dyn(r: &ChannelDescriptor) -> SwizzleResultDyn {
    Ok(DynamicImage::ImageLuma8(to_luma(r)?))
}

/// Generates a _Luminance Alpha_ image from two descriptors
///
/// **NOTE**: not all descriptors are required. When no descriptor is provided
/// for a channel, the channel is left empty.
///
/// # Arguments
///
/// * `r` - The descriptor to use for writing the _red_ channel
/// * `a` - The descriptor to use for writing the _alpha_ channel
///
/// # Example
///
/// ```
/// let descriptor_r = ChannelDescriptor::from_path(PathBuf::from("./input.png"), 0);
/// let descriptor_a = ChannelDescriptor::from_path(PathBuf::from("./input2.png"), 0);
/// let image = to_luma(&Some(descriptor_r), &Some(descriptor_a));
/// ```
pub fn to_luma_a(
    r: &Option<ChannelDescriptor>,
    a: &Option<ChannelDescriptor>,
) -> SwizzleResult<image::GrayAlphaImage> {
    static DEFAULT: LumaA<u8> = LumaA([0, 255]);
    swizzle!(DEFAULT, r, a)
}

/// Generates a _Luminance Alpha_ image from two descriptors, and wraps it
/// into a `image::DynamicImage`
///
/// **NOTE**: not all descriptors are required. When no descriptor is provided
/// for a channel, the channel is left empty.
///
/// # Arguments
///
/// * `r` - The descriptor to use for writing the _red_ channel
/// * `a` - The descriptor to use for writing the _alpha_ channel
pub fn to_luma_a_dyn(
    r: &Option<ChannelDescriptor>,
    g: &Option<ChannelDescriptor>,
) -> SwizzleResultDyn {
    Ok(DynamicImage::ImageLumaA8(to_luma_a(r, g)?))
}

/// Generates a _RGB_ image from three descriptors
///
/// **NOTE**: not all descriptors are required. When no descriptor is provided
/// for a channel, the channel is left empty.
///
/// # Arguments
///
/// * `r` - The descriptor to use for writing the _red_ channel
/// * `g` - The descriptor to use for writing the _green_ channel
/// * `b` - The descriptor to use for writing the _blue_ channel
///
/// # Example
///
/// ```
/// use std::path::PathBuf;
///
/// let descriptor_r = ChannelDescriptor::from_path(PathBuf::from("./input.png"), 0);
/// let descriptor_g = ChannelDescriptor::from_path(PathBuf::from("./input2.png"), 0);
/// let descriptor_b = ChannelDescriptor::from_path(PathBuf::from("./input3.png"), 0);
/// let image = to_rgb(
///   &Some(descriptor_r),
///   &Some(descriptor_g),
///   &Some(descriptor_b)
/// );
/// ```
pub fn to_rgb(
    r: &Option<ChannelDescriptor>,
    g: &Option<ChannelDescriptor>,
    b: &Option<ChannelDescriptor>,
) -> SwizzleResult<image::RgbImage> {
    static DEFAULT: Rgb<u8> = Rgb([0, 0, 0]);
    swizzle!(DEFAULT, r, g, b)
}

/// Generates a _RGB_ image from three descriptors, and wraps it
/// into a `image::DynamicImage`
///
/// **NOTE**: not all descriptors are required. When no descriptor is provided
/// for a channel, the channel is left empty.
///
/// # Arguments
///
/// * `r` - The descriptor to use for writing the _red_ channel
/// * `g` - The descriptor to use for writing the _green_ channel
/// * `b` - The descriptor to use for writing the _blue_ channel
pub fn to_rgb_dyn(
    r: &Option<ChannelDescriptor>,
    g: &Option<ChannelDescriptor>,
    b: &Option<ChannelDescriptor>,
) -> SwizzleResultDyn {
    Ok(DynamicImage::ImageRgb8(to_rgb(r, g, b)?))
}

/// Generates a _RGBA_ image from four descriptors.
///
/// **NOTE**: not all descriptors are required. When no descriptor is provided
/// for a channel, the channel is left empty.
///
/// # Arguments
///
/// * `r` - The descriptor to use for writing the _red_ channel
/// * `g` - The descriptor to use for writing the _green_ channel
/// * `b` - The descriptor to use for writing the _blue_ channel
/// * `a` - The descriptor to use for writing the _alpha_ channel
///
/// # Example
///
/// ```
/// use std::path::PathBuf;
///
/// let descriptor_r = ChannelDescriptor::from_path(PathBuf::from("./input.png"), 0);
/// let descriptor_g = ChannelDescriptor::from_path(PathBuf::from("./input2.png"), 0);
/// let descriptor_b = ChannelDescriptor::from_path(PathBuf::from("./input3.png"), 0);
/// let descriptor_a = ChannelDescriptor::from_path(PathBuf::from("./input4.png"), 0);
/// let image = to_rgba(
///   &Some(descriptor_r),
///   &Some(descriptor_g),
///   &Some(descriptor_b),
///   &Some(descriptor_a)
/// );
/// ```
pub fn to_rgba(
    r: &Option<ChannelDescriptor>,
    g: &Option<ChannelDescriptor>,
    b: &Option<ChannelDescriptor>,
    a: &Option<ChannelDescriptor>,
) -> SwizzleResult<image::RgbaImage> {
    static DEFAULT: Rgba<u8> = Rgba([0, 0, 0, 255]);
    swizzle!(DEFAULT, r, g, b, a)
}

/// Generates a _RGBA_ image from three descriptors, and wraps it
/// into a `image::DynamicImage`
///
/// **NOTE**: not all descriptors are required. When no descriptor is provided
/// for a channel, the channel is left empty.
///
/// # Arguments
///
/// * `r` - The descriptor to use for writing the _red_ channel
/// * `g` - The descriptor to use for writing the _green_ channel
/// * `b` - The descriptor to use for writing the _blue_ channel
/// * `a` - The descriptor to use for writing the _alpha_ channel
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
        1 => {
            match &descriptors[0] {
                Some(d) => image::DynamicImage::ImageLuma8(to_luma(d)?),
                None => return Err(ErrorKind::EmptyDescriptor)
            }
        },
        2 => image::DynamicImage::ImageLumaA8(to_luma_a(&descriptors[0], &descriptors[1])?),
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
