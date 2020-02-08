use structopt::StructOpt;

use swizzler::{
    errors::ErrorKind,
    ChannelDescriptor,
    to_rgba,
    to_rgb,
    to_luma,
    to_lumaA,
    to_dynamic
};

use image::{
    DynamicImage,
    RgbaImage,
    open
};

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct ManualCommand {

    #[structopt(long = "input", short)]
    descriptors: Vec<String>,

    #[structopt(
        long = "output",
        short,
        parse(from_os_str),
        default_value = "output.png"
    )]
    output: std::path::PathBuf

}

#[derive(StructOpt)]
enum Command {

    Manual(ManualCommand)

}

#[derive(StructOpt)]
#[structopt(
    name = "swizzler-cli",
    about = "Swizzle images components intp a single output",
    rename_all = "kebab-case"
)]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command
}

fn process_manual(command: &ManualCommand) -> Result<(), ErrorKind> {
    let descriptors: Vec<Option<ChannelDescriptor>> =
        (command.descriptors
            .iter()
            .map(|s| -> Result<Option<ChannelDescriptor>, ErrorKind> {
                Ok(Some(ChannelDescriptor::from_path(&s)?))
            })
            .collect::<Result<Vec<Option<ChannelDescriptor>>, ErrorKind>>()
        )?;

    let image = to_dynamic(&descriptors)?;
    image.save(&command.output)?;
    Ok(())
}

fn main() -> Result<(), ErrorKind> {
    let args = Opt::from_args();

    match &args.cmd {
        Command::Manual(manual) => {
            process_manual(&manual)
        },
        _ => Ok(())
    }

}
