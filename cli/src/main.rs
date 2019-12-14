use std::path::PathBuf;
use structopt::StructOpt;

use texture_packer;
use texture_packer::{
    Error
};

#[derive(StructOpt)]
#[structopt(
    name = "texture-synthesis",
    about = "Synthesizes images based on example images",
    rename_all = "kebab-case"
)]
struct Opt {
    #[structopt(long = "input", short, parse(from_os_str))]
    input: PathBuf,
    #[structopt(long = "output", short, parse(from_os_str))]
    output: PathBuf,
}

fn process() -> Result<(), texture_packer::Error> {
    let img = texture_packer::load_image(&String::from("./cat.png"));
    match img {
        Ok(i) => Ok(println!("YESSSS NOICE")),
        Err(e) => Err(Error::Image(e)),
    }
}

fn main() {
    let args = Opt::from_args();
    if let Err(e) = process() {
        if atty::is(atty::Stream::Stderr) {
            eprintln!("\x1b[31merror\x1b[0m: {}", "noees");
        } else {
            eprintln!("error: {}", "noees");
        }
        std::process::exit(1);
    }
}
