use image::{
    DynamicImage
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

    fn get_filename() -> String;

}

pub struct GenericTarget {

    inputs: Vec<Option<(String, u8)>>

}

impl GenericTarget {

    pub fn new(inputs: Vec<Option<(String, u8)>>) -> GenericTarget {
        GenericTarget { inputs }
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

    fn get_filename() -> String {
        String::new()
    }

}

pub struct GenericWriter {

    pub targets: Vec<GenericTarget>

}

impl GenericWriter {

    pub fn new(targets: Vec<GenericTarget>) -> GenericWriter {
        GenericWriter { targets }
    }

}
