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
/// use swizzler::ChannelDescriptor;
///
/// // Creates a descriptor pointing to the **red** channel of the `input.png` image.
/// let descriptor = ChannelDescriptor::from_path(PathBuf::from("./input.png"), 0);
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

    /// Generates a descriptor from an image RC pointer and a channel.
    ///
    /// # Arguments
    ///
    /// * `img_input` - RC pointer to the image source
    /// * `channel` - Source channel in the given input source
    pub fn from_image_rc(
        img_input: &std::rc::Rc<image::DynamicImage>,
        channel: u8,
    ) -> ChannelDescResult {
        let img = std::rc::Rc::clone(img_input);
        Ok(ChannelDescriptor { img: img, channel })
    }

    /// Generates a descriptor from an image and a channel.
    ///
    /// # Arguments
    ///
    /// * `img_input` - Image source
    /// * `channel` - Source channel in the given input source
    pub fn from_image(img_input: image::DynamicImage, channel: u8) -> ChannelDescResult {
        let img = std::rc::Rc::new(img_input);
        Ok(ChannelDescriptor { img, channel })
    }

    /// Generates a descriptor from a path and a channel.
    ///
    /// # Arguments
    ///
    /// * `img_input` - Image source
    /// * `channel` - Source channel in the given input source
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use swizzler::ChannelDescriptor;
    ///
    /// // Creates a descriptor pointing to file "./input.png", and set it up
    /// // to read its `red` channel (channel 0).
    /// let descriptor = ChannelDescriptor::from_path(PathBuf::from("./input.png"), 0);
    /// ```
    pub fn from_path<T>(path: T, channel: u8) -> ChannelDescResult
    where
        T: AsRef<std::path::Path>,
    {
        let img = image::open(path.as_ref())?;
        ChannelDescriptor::from_image(img, channel)
    }

    /// Generates a descriptor from a string containing the path to the image
    /// source, to which is appended the channel to read.
    ///
    /// # Arguments
    ///
    /// * `input` - String containing the path to the image, followed by the
    /// separator `:` and the channel to read
    ///
    /// # Examples
    ///
    /// ```
    /// use swizzler::ChannelDescriptor;
    ///
    /// // Creates a descriptor pointing to file "./input.png", and set it up
    /// // to read its `red` channel (channel 0).
    /// let descriptor = ChannelDescriptor::from_description("./input.png:0");
    /// ```
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

#[cfg(test)]
mod tests {

    use image::{DynamicImage, GrayImage, GrayAlphaImage, ImageBuffer, Luma, LumaA, Rgb, Rgba, RgbImage, RgbaImage};
    use crate::swizzle::{ChannelDescriptor, to_luma, to_luma_a, to_rgb, to_rgba};

    fn assert_pixels<P: image::Pixel, Container>(
        img: &ImageBuffer<P, Container>,
        expected: &[ P ]
    ) where
        P: std::cmp::PartialEq + std::fmt::Debug + 'static,
        Container: std::ops::Deref<Target = [P::Subpixel]>
    {
        let (width, _) = img.dimensions();
        for (i, e) in expected.iter().enumerate() {
            let x = (i as u32) % width;
            let y = (i as u32) / width;
            assert_eq!(
                *img.get_pixel(x, y),
                *e,
                "pixel comparison failed at ({}, {})", x, y
            );
        }
    }

    #[test]
    fn swizzle_grayscale() {
        let mut img: RgbImage = ImageBuffer::new(2, 2);
        img.put_pixel(0, 0, Rgb([128, 128, 128]));
        img.put_pixel(1, 0, Rgb([0, 135, 97]));
        img.put_pixel(0, 1, Rgb([255, 78, 23]));
        img.put_pixel(1, 1, Rgb([100, 0, 255]));
        let img = std::rc::Rc::new(DynamicImage::ImageRgb8(img));

        // Test with a `red` descriptor
        let descriptor = ChannelDescriptor::from_image_rc(&img, 0).unwrap();
        let result = to_luma(&descriptor).unwrap();
        assert_eq!(result.dimensions(), (2, 2));
        assert_pixels(&result, &[ Luma([128]), Luma([0]), Luma([255]), Luma([100]) ]);

        // Test with a `green` descriptor
        let descriptor = ChannelDescriptor::from_image_rc(&img, 1).unwrap();
        let result = to_luma(&descriptor).unwrap();
        assert_pixels(&result, &[ Luma([128]), Luma([135]), Luma([78]), Luma([0]) ]);

        // Test with a `blue` descriptor
        let descriptor = ChannelDescriptor::from_image_rc(&img, 2).unwrap();
        let result = to_luma(&descriptor).unwrap();
        assert_pixels(&result, &[ Luma([128]), Luma([97]), Luma([23]), Luma([255]) ]);
    }

    #[test]
    fn swizzle_luma_a() {
        let mut img: GrayAlphaImage = ImageBuffer::new(2, 1);
        img.put_pixel(0, 0, LumaA([0, 250]));
        img.put_pixel(1, 0, LumaA([129, 13]));
        let img = std::rc::Rc::new(DynamicImage::ImageLumaA8(img));

        // Test with a `red` descriptor
        let descriptor_r = Some(ChannelDescriptor::from_image_rc(&img, 1).unwrap());
        let result = to_luma_a(&descriptor_r, &None).unwrap();
        assert_eq!(result.dimensions(), (2, 1));
        assert_pixels(&result, &[ LumaA([250, 255]),  LumaA([13, 255]) ]);

        // Test with a `red` + `alpha` descriptors
        let descriptor_a = Some(ChannelDescriptor::from_image_rc(&img, 0).unwrap());
        let result = to_luma_a(&descriptor_r, &descriptor_a).unwrap();
        assert_pixels(&result, &[ LumaA([250, 0]),  LumaA([13, 129]) ]);
    }

    #[test]
    fn swizzle_rgb() {
        let mut img: RgbaImage = ImageBuffer::new(2, 1);
        img.put_pixel(0, 0, Rgba([1, 2, 3, 255]));
        img.put_pixel(1, 0, Rgba([127, 128, 126, 0]));
        let img = std::rc::Rc::new(DynamicImage::ImageRgba8(img));

        // Test to set only the `green` channel
        let result = to_rgb(
            &None,
            &Some(ChannelDescriptor::from_image_rc(&img, 3).unwrap()),
            &None
        ).unwrap();
        assert_eq!(result.dimensions(), (2, 1));
        assert_pixels(&result, &[ Rgb([0, 255, 0]), Rgb([0, 0, 0]) ]);

        // Test to set all channels
        let result = to_rgb(
            &Some(ChannelDescriptor::from_image_rc(&img, 2).unwrap()),
            &Some(ChannelDescriptor::from_image_rc(&img, 1).unwrap()),
            &Some(ChannelDescriptor::from_image_rc(&img, 0).unwrap())
        ).unwrap();
        assert_pixels(&result, &[ Rgb([3, 2, 1]), Rgb([126, 128, 127]) ]);
    }

    #[test]
    fn swizzle_rgba() {
        let mut img: RgbaImage = ImageBuffer::new(2, 1);
        img.put_pixel(0, 0, Rgba([255, 128, 0, 0]));
        img.put_pixel(1, 0, Rgba([0, 128, 255, 255]));
        let img = std::rc::Rc::new(DynamicImage::ImageRgba8(img));

        // Test to set the `red` and `alpha` channels
        let result = to_rgba(
            &Some(ChannelDescriptor::from_image_rc(&img, 0).unwrap()),
            &None,
            &None,
            &Some(ChannelDescriptor::from_image_rc(&img, 3).unwrap())
        ).unwrap();
        assert_eq!(result.dimensions(), (2, 1));
        assert_pixels(&result, &[ Rgba([255, 0, 0, 0]), Rgba([0, 0, 0, 255]) ]);

        // Test to set all channels
        let result = to_rgba(
            &Some(ChannelDescriptor::from_image_rc(&img, 3).unwrap()),
            &Some(ChannelDescriptor::from_image_rc(&img, 2).unwrap()),
            &Some(ChannelDescriptor::from_image_rc(&img, 1).unwrap()),
            &Some(ChannelDescriptor::from_image_rc(&img, 0).unwrap())
        ).unwrap();
        assert_pixels(&result, &[ Rgba([0, 0, 128, 255]), Rgba([255, 255, 128, 0]) ]);
    }

    #[test]
    fn swizzle_rgba_multisources() {
        let mut img_1: GrayAlphaImage = ImageBuffer::new(2, 1);
        img_1.put_pixel(0, 0, LumaA([77, 128]));
        img_1.put_pixel(1, 0, LumaA([255, 0]));
        let img_1 = DynamicImage::ImageLumaA8(img_1);

        let mut img_2: RgbaImage = ImageBuffer::new(2, 1);
        img_2.put_pixel(0, 0, Rgba([ 128, 129, 130, 131 ]));
        img_2.put_pixel(1, 0, Rgba([ 42, 40, 12, 132 ]));
        let img_2 = DynamicImage::ImageRgba8(img_2);

        let mut img_3: GrayImage = ImageBuffer::new(2, 1);
        img_3.put_pixel(0, 0, Luma([ 1 ]));
        img_3.put_pixel(1, 0, Luma([ 2 ]));
        let img_3 = DynamicImage::ImageLuma8(img_3);

        let mut img_4: RgbImage = ImageBuffer::new(2, 1);
        img_4.put_pixel(0, 0, Rgb([78, 79, 80]));
        img_4.put_pixel(1, 0, Rgb([5, 6, 7]));
        let img_4 = DynamicImage::ImageRgb8(img_4);

        let result = to_rgba(
            &Some(ChannelDescriptor::from_image(img_1, 1).unwrap()),
            &Some(ChannelDescriptor::from_image(img_2, 3).unwrap()),
            &Some(ChannelDescriptor::from_image(img_3, 0).unwrap()),
            &Some(ChannelDescriptor::from_image(img_4, 2).unwrap())
        ).unwrap();
        assert_pixels(&result, &[ Rgba([128, 131, 1, 80]), Rgba([0, 132, 2, 7]) ]);
    }

    #[test]
    fn use_non_matching_dimensions() {
        let mut img_1: GrayImage = ImageBuffer::new(2, 1);
        img_1.put_pixel(0, 0, Luma([ 1 ]));
        img_1.put_pixel(1, 0, Luma([ 2 ]));
        let img_1 = DynamicImage::ImageLuma8(img_1);

        let mut img_2: RgbImage = ImageBuffer::new(1, 1);
        img_2.put_pixel(0, 0, Rgb([0, 0, 0]));
        let img_2 = DynamicImage::ImageRgb8(img_2);

        let result = to_rgba(
            &Some(ChannelDescriptor::from_image(img_1, 1).unwrap()),
            &Some(ChannelDescriptor::from_image(img_2, 3).unwrap()),
            &None,
            &None
        );

        assert!(result.is_err(), "result should because of invalid dimensions");
    }

}
