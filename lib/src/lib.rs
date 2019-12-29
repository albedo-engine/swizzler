pub use errors::Error;

mod errors;
use image;
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

#[derive(Copy, Clone)]
pub enum Channel {
    R = 0,
    G,
    B,
    A
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

pub fn load_image<T>(source: &T) -> ImageResult where T: ImageLoader {
    source.create_image()
}

pub fn process<'a>(
    out: &mut image::DynamicImage,
    commands: &'a Vec<Command>
) -> Result<(), errors::Error> {

    let out_buffer = match out.as_mut_rgb8() {
        Some(buffer) => buffer,
        _ => return Err(Error::Invalid),
    };
    let (width, height) = out_buffer.dimensions();

    for c in commands {
        if let Some(buffer) = c.img.as_rgb8() {
            println!("{}", out_buffer.pixels_mut().len());
            for (i, pix) in out_buffer.pixels_mut().enumerate() {
                let p = i as u32;
                let x = p % width as u32;
                let y = p / width as u32;
                let pixel_read = buffer.get_pixel(x, y);
                (*pix)[c.output_channel as usize] = (*pixel_read)[c.output_channel as usize];
                // *pix = *pixel_read;
            }
        }
    };
    Ok(())
}
