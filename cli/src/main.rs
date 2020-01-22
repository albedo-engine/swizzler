use std::path::PathBuf;
use std::process;
use std::collections::HashMap;
use structopt::StructOpt;
use std::str::FromStr;

use texture_packer::{
    errors::ErrorKind,
    Swizzle,
    ChannelDescriptor
};

use image::{
    DynamicImage,
    RgbaImage,
    open
};

#[derive(StructOpt)]
#[structopt(
    name = "texture-synthesis",
    about = "Synthesizes images based on example images",
    rename_all = "kebab-case"
)]
struct Opt {
    /*#[structopt(long = "input", short)]
    input: Vec<String>,

    #[structopt(long = "output", short, parse(from_os_str))]
    output: PathBuf,*/
}

struct ImagesPool {

    images: HashMap<String, DynamicImage>

}

impl ImagesPool {

    fn new() -> ImagesPool {
        ImagesPool {
            images: HashMap::new()
        }
    }

    fn load<P>(&mut self, path: P)
        -> Result<(), ErrorKind> where P: AsRef<str> {
        let p = path.as_ref();
        if !self.images.contains_key(p) {
            let img = open(path.as_ref())?;
            self.images.insert(String::from(p), img);
        }
        Ok(())
    }

    fn get<P>(&self, path: P) -> Option<&DynamicImage> where P: AsRef<str> {
        self.images.get(path.as_ref())
    }

}

fn main() -> Result<(), ErrorKind> {
    let args = Opt::from_args();

    let mut img_pool = ImagesPool::new();
    img_pool.load("./cat.png")?;

    let result = RgbaImage::swizzle([
        Some(ChannelDescriptor::new(img_pool.get("./cat.png").unwrap(), 2)),
        Some(ChannelDescriptor::new(img_pool.get("./cat.png").unwrap(), 1)),
        Some(ChannelDescriptor::new(img_pool.get("./cat.png").unwrap(), 0)),
        Some(ChannelDescriptor::new(img_pool.get("./cat.png").unwrap(), 0))
    ]).unwrap();

    result.save("./output.png")?;

    Ok(())

    /* let input_imgs: Result<Vec<_>, &'static str> = args.input
        .iter()
        .map(|s| InputImage::from_str(s))
        .collect();

    let input_imgs: Vec<InputImage> = input_imgs.unwrap_or_else(|e| {
        println!("Problem parsing arguments: {}", e);
        process::exit(1);
    });

    let mut session = Session::new();
    for input in input_imgs {
        session.add_input(input)?;
    }
    let img = session.run((1200, 600))?;
    img.save("./output.png").unwrap(); */

}
