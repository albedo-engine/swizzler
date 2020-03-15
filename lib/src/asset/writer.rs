use image::{
    DynamicImage,
    ImageFormat
};

use crate::swizzler::{
    to_rgba_dyn,
    ChannelDescriptor
};

use crate::errors::{
    ErrorKind
};

use crate::asset::{
    GenericAsset
};

pub trait Target {

    fn generate(&self, asset: &GenericAsset) -> Result<DynamicImage, ErrorKind>;

    fn get_filename(&self, asset: &GenericAsset) -> String;

}

pub struct GenericTarget {

    pub output_format: image::ImageFormat,

    pub inputs: Vec<Option<(String, u8)>>

}

impl GenericTarget {

    pub fn new(inputs: Vec<Option<(String, u8)>>) -> GenericTarget {
        GenericTarget {
            output_format: image::ImageFormat::PNG,
            inputs
        }
    }

    fn _create_descriptor(
        &self,
        index: usize,
        asset: &GenericAsset
    ) -> Result<Option<ChannelDescriptor>, ErrorKind> {
        if let Some(input) = &self.inputs[index] {
            match asset.get_texture_path(&input.0) {
                Some(path) => Ok(Some(ChannelDescriptor::from_path(path, input.1)?)),
                _ => Ok(None)
            }
        } else {
            Ok(None)
        }
    }

}

impl Target for GenericTarget {

    fn generate(&self, asset: &GenericAsset) -> Result<DynamicImage, ErrorKind> {
        match self.inputs.len() {
            1 => Err(ErrorKind::NoInputs),
            2 => Err(ErrorKind::NoInputs),
            3 => Err(ErrorKind::NoInputs),
            a if a >= 4 => {
                to_rgba_dyn(
                    &self._create_descriptor(0, asset)?,
                    &self._create_descriptor(1, asset)?,
                    &self._create_descriptor(2, asset)?,
                    &self._create_descriptor(3, asset)?
                )
            },
            _ => panic!("too big vector!")
        }
    }

    fn get_filename(&self, asset: &GenericAsset) -> String {
        let mut result = String::from(asset.get_base());
        result.push_str("-");
        for input in &self.inputs {
            if let Some(input) = input {
                result.push_str(&input.0);
            }
        }
        result.push_str(".");
        result.push_str(get_image_format_ext(self.output_format));
        result
    }

}

pub struct GenericWriter {

    pub targets: Vec<GenericTarget>

}

impl GenericWriter {

    pub fn new(targets: Vec<GenericTarget>) -> GenericWriter {
        GenericWriter {
            targets
        }
    }

}

fn get_image_format_ext(format: ImageFormat) -> &'static str {
    match &format {
        ImageFormat::PNG => "png",
        ImageFormat::JPEG => "jpg",
        ImageFormat::TIFF => "tif",
        ImageFormat::TGA => "tga",
        ImageFormat::HDR => "hdr",
        ImageFormat::GIF => "gif",
        ImageFormat::BMP => "bpm",
        ImageFormat::WEBP => "webp",
        ImageFormat::ICO => "ico",
        ImageFormat::PNM => "pnm"
    }
}
