use std::path::PathBuf;
use std::process;
use std::collections::HashMap;
use std::str::FromStr;
use structopt::StructOpt;
use image;
use image::GenericImageView;

use texture_packer;
use texture_packer::{
    Command,
    Channel,
    Error
};

struct InputImage {
    channel: Channel,
    img_path: String,
}

impl FromStr for InputImage {

    type Err = &'static str;

    fn from_str(input: &str) -> Result<InputImage, Self::Err> {
        let mut split = input.split(':');
        let img_path = String::from(split.next().ok_or("error")?);
        let last = split.next().ok_or("error")?;
        let channel = match &last.to_lowercase()[..] {
            "r" => Ok(Channel::R),
            "g" => Ok(Channel::G),
            "b" => Ok(Channel::B),
            "a" => Ok(Channel::A),
            _ => Err("failed")
        }.unwrap();
        Ok(InputImage { channel, img_path })
    }

}

#[derive(StructOpt)]
#[structopt(
    name = "texture-synthesis",
    about = "Synthesizes images based on example images",
    rename_all = "kebab-case"
)]
struct Opt {
    #[structopt(long = "input", short)]
    input: Vec<String>,

    #[structopt(long = "output", short, parse(from_os_str))]
    output: PathBuf,
}

fn process(
    out_img: &mut image::DynamicImage,
    out_name: &PathBuf,
    cmds: &Vec<Command>
) -> Result<(), texture_packer::Error> {
    texture_packer::process(out_img, &cmds)?;
    out_img.save(out_name)?;
    Ok(())
}

fn create_image_map(input: &Vec<InputImage>) ->
    Result<HashMap<String, image::DynamicImage>, texture_packer::Error> {
    let mut imgs_map: HashMap<String, image::DynamicImage> = HashMap::new();
    imgs_map.reserve(input.len());
    for c in input {
        if !imgs_map.contains_key(&c.img_path) {
            let img = texture_packer::load_image(&String::from("./cat.png"))?;
            imgs_map.insert(c.img_path.clone(), img);
        }
    }
    Ok(imgs_map)
}

fn build_commands<'a>(
    input: &Vec<InputImage>,
    images_map: &'a HashMap<String, image::DynamicImage>
) -> Vec<Command<'a>> {
    let mut cmds: Vec<Command> = Vec::new();
    cmds.reserve(input.len());

    for i in input {
        let image = images_map.get(&i.img_path).unwrap();
        cmds.push(Command::new(image, i.channel, i.channel, None));
    }

    cmds
}

fn main() {
    let args = Opt::from_args();

    let input_imgs: Result<Vec<_>, &'static str> = args.input
        .iter()
        .map(|s| InputImage::from_str(s))
        .collect();

    let input_imgs: Vec<InputImage> = input_imgs.unwrap_or_else(|e| {
        println!("Problem parsing arguments: {}", e);
        process::exit(1);
    });

    let images_map = create_image_map(&input_imgs).unwrap_or_else(|e| {
        println!("Problem loading an image");
        process::exit(1);
    });

    let ( width, height ) = images_map.values().next().unwrap().dimensions();

    let mut result_img = image::DynamicImage::new_rgb8(width, height);
    let cmds: Vec<Command> = build_commands(&input_imgs, &images_map);

    if let Err(e) = process(&mut result_img, &args.output, &cmds) {
        if atty::is(atty::Stream::Stderr) {
            eprintln!("\x1b[31merror\x1b[0m: {}", e);
        } else {
            eprintln!("error: {}", e);
        }
        std::process::exit(1);
    }
}
