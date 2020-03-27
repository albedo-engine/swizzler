use image::{ImageBuffer, GrayAlphaImage, RgbImage, RgbaImage, DynamicImage, Rgb, Luma, LumaA, Rgba};
use swizzler::{to_luma, to_luma_a, to_rgb, ChannelDescriptor};

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
    assert_eq!(*result.get_pixel(0, 0), Luma([128]));
    assert_eq!(*result.get_pixel(1, 0), Luma([0]));
    assert_eq!(*result.get_pixel(0, 1), Luma([255]));
    assert_eq!(*result.get_pixel(1, 1), Luma([100]));

    // Test with a `green` descriptor
    let descriptor = ChannelDescriptor::from_image_rc(&img, 1).unwrap();
    let result = to_luma(&descriptor).unwrap();
    assert_eq!(*result.get_pixel(0, 0), Luma([128]));
    assert_eq!(*result.get_pixel(1, 0), Luma([135]));
    assert_eq!(*result.get_pixel(0, 1), Luma([78]));
    assert_eq!(*result.get_pixel(1, 1), Luma([0]));

    // Test with a `blue` descriptor
    let descriptor = ChannelDescriptor::from_image_rc(&img, 2).unwrap();
    let result = to_luma(&descriptor).unwrap();
    assert_eq!(*result.get_pixel(0, 0), Luma([128]));
    assert_eq!(*result.get_pixel(1, 0), Luma([97]));
    assert_eq!(*result.get_pixel(0, 1), Luma([23]));
    assert_eq!(*result.get_pixel(1, 1), Luma([255]));
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
    assert_eq!(*result.get_pixel(0, 0), LumaA([250, 255]));
    assert_eq!(*result.get_pixel(1, 0), LumaA([13, 255]));

    // Test with a `red` + `alpha` descriptors
    let descriptor_a = Some(ChannelDescriptor::from_image_rc(&img, 0).unwrap());
    let result = to_luma_a(&descriptor_r, &descriptor_a).unwrap();
    assert_eq!(*result.get_pixel(0, 0), LumaA([250, 0]));
    assert_eq!(*result.get_pixel(1, 0), LumaA([13, 129]));
}

#[test]
fn swizzle_rgb() {
    let mut img: RgbaImage = ImageBuffer::new(2, 1);
    img.put_pixel(0, 0, Rgba([1, 2, 3, 255]));
    img.put_pixel(1, 0, Rgba([127, 128, 126, 0]));
    let img = std::rc::Rc::new(DynamicImage::ImageRgba8(img));

    // Test with a `green` descriptor only
    let result = to_rgb(
        &None,
        &Some(ChannelDescriptor::from_image_rc(&img, 3).unwrap()),
        &None
    ).unwrap();
    assert_eq!(result.dimensions(), (2, 1));
    assert_eq!(*result.get_pixel(0, 0), Rgb([0, 255, 0]));
    assert_eq!(*result.get_pixel(1, 0), Rgb([0, 0, 0]));

    // Test with a `green` descriptor only
    let result = to_rgb(
        &Some(ChannelDescriptor::from_image_rc(&img, 2).unwrap()),
        &Some(ChannelDescriptor::from_image_rc(&img, 1).unwrap()),
        &Some(ChannelDescriptor::from_image_rc(&img, 0).unwrap())
    ).unwrap();
    assert_eq!(result.dimensions(), (2, 1));
    assert_eq!(*result.get_pixel(0, 0), Rgb([3, 2, 1]));
    assert_eq!(*result.get_pixel(1, 0), Rgb([126, 128, 127]));
}
