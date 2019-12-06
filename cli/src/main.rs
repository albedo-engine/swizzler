use std::path::PathBuf;
use structopt::StructOpt;

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
        (Some(x), Some(y)) => println!("{} and {}", x, y),
        _ => ()
    }
}
