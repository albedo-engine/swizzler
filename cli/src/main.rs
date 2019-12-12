use std::path::PathBuf;
use structopt::StructOpt;

use texture_packer;

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

fn main() {
    let args = Opt::from_args();
    let test = (args.input.to_str(), args.output.to_str());
    match test {
        (Some(x), Some(y)) => process(),
        _ => (),
    }
}

fn process() -> () {
    if let Err(e) = texture_packer::exec() {
        if atty::is(atty::Stream::Stderr) {
            eprintln!("\x1b[31merror\x1b[0m: {}", e);
        } else {
            eprintln!("error: {}", e);
        }
        std::process::exit(1);
    }
}
