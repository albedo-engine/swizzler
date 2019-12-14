mod errors;
use std::path::Path;

pub type ImageResult = image::ImageResult<image::DynamicImage>;

pub trait ImageLoader {

    fn create_image(&self) -> ImageResult;

}

impl ImageLoader for String {
    fn create_image(&self) -> ImageResult {
        image::open(self)
    }
}

impl ImageLoader for Path {

    fn create_image(&self) -> ImageResult {
        image::open(self)
    }

}

/* impl ImageLoader for [u8] {

    fn create_image(&self) -> ImageResult {
        image::load_from_memory(self);
    }

} */

pub enum Channel {
    R, G, B, A
}

pub enum ProcessStep {
    Inverse,
}

pub struct Command<'a> {
    img: &'a image::DynamicImage,
    input_channel: Channel,
    output_channel: Channel,
    process_step: Option<ProcessStep>,
}

impl Command<'_> {

   pub fn new(
        img: &image::DynamicImage,
        input_channel: Channel,
        output_channel: Channel,
        process_step: Option<ProcessStep>
    ) -> Command {
        Command {
            img,
            input_channel,
            output_channel,
            process_step,
        }
    }

}

pub use errors::Error;

pub fn load_image<T>(source: &T) -> ImageResult where T: ImageLoader {
    source.create_image()
}

fn exec_internal(mut img: image::DynamicImage) -> () {
    // The dimensions method returns the images width and height.
    // println!("dimensions {:?}", img.dimensions());

    if let Some(buffer) = img.as_mut_rgb8() {
        for (i, pix) in buffer.pixels_mut().enumerate() {
            (*pix)[0] = 255 - (*pix)[0];
            (*pix)[1] = 255 - (*pix)[1];
            (*pix)[2] = 255 - (*pix)[2];
        }
        img.save("./cat-output.png");
    }
}

pub fn exec() -> Result<(), image::ImageError> {

    match image::open("./cat.png") {
        Ok(x) => Ok(exec_internal(x)),
        Err(e) => Err(e)
    }

}
