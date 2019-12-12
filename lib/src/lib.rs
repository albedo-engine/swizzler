pub use image;

enum Channel {
    R, G, B, A
}

struct CommandOption {
    input_channel: Channel,
    output_channel: Channel,
    invert_input: bool
}

trait Command {

    fn run(
        mut out: image::DynamicImage,
        input: image::DynamicImage,
        option: CommandOption
    ) -> ();

}

fn process(mut img: image::ImageBuffer) -> () {

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
