use regex;
use std::io::Read;
use structopt::StructOpt;

use swizzler::session::{resolve_assets_dir, GenericAssetReader, Session};
use swizzler::{errors::ErrorKind, to_dynamic, ChannelDescriptor};

mod json;
use json::{parse_image_format, Config};

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
/// Struct containing the parsed configuration for a `session` StructOpt
/// command.
struct ManualCommand {
    #[structopt(long = "input", short = "i")]
    inputs: Vec<String>,

    #[structopt(
        long = "output",
        short,
        parse(from_os_str),
        default_value = "output.png"
    )]
    output: std::path::PathBuf,

    #[structopt(long = "format", short, parse(try_from_str = parse_image_format))]
    format: Option<image::ImageFormat>,
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
/// Struct containing the parsed configuration for a `session` StructOpt
/// command.
struct SessionCommand {
    #[structopt(long = "folder", short, parse(from_os_str))]
    folder: std::path::PathBuf,

    #[structopt(long = "config", short, parse(from_os_str))]
    config: Option<std::path::PathBuf>,

    #[structopt(long = "threads", short)]
    num_threads: Option<usize>,

    #[structopt(
        long = "output",
        short,
        parse(from_os_str),
        default_value = "./__swizzler_build"
    )]
    output: std::path::PathBuf,
}

#[derive(StructOpt)]
enum Command {
    Manual(ManualCommand),
    Session(SessionCommand),
}

#[derive(StructOpt)]
#[structopt(
    name = "swizzler-cli",
    about = "Swizzle multiple source images into a single image output",
    rename_all = "kebab-case"
)]
struct Opt {
    #[structopt(short)]
    quiet: bool,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug)]
pub enum CLIError {
    SwizzlerError(ErrorKind),
    JSONError(serde_json::Error),
    IOError(std::io::Error),
    MissingInput,
}

impl std::fmt::Display for CLIError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            CLIError::JSONError(s) => write!(f, "json parsing failed: '{}'", s),
            CLIError::IOError(e) => write!(f, "config file couldn't be loaded: {}", e),
            CLIError::MissingInput => write!(f, "no inputs provided"),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl From<ErrorKind> for CLIError {
    fn from(e: ErrorKind) -> Self {
        Self::SwizzlerError(e)
    }
}

impl From<serde_json::Error> for CLIError {
    fn from(e: serde_json::Error) -> Self {
        Self::JSONError(e)
    }
}

impl From<std::io::Error> for CLIError {
    fn from(ie: std::io::Error) -> Self {
        println!("Called");
        Self::IOError(ie)
    }
}

macro_rules! log {
    ( $quiet:expr, $msg:expr ) => {
        if !$quiet {
            println!("{}", $msg);
        }
    };
}

/// Executes a manual command.
///
/// A manual command takes up to four input images, and swizzle their channels
/// into a new image. This allows user to swizzle anything using this CLI.
fn process_manual(command: &ManualCommand, quiet: bool) -> Result<(), CLIError> {
    let none_regex = regex::Regex::new(r"(?i)none").unwrap();
    // Converts inputs into channel descriptors, that the Swizzler library
    // can use to generate the image.
    let descriptors: Vec<Option<ChannelDescriptor>> = (command
        .inputs
        .iter()
        .map(|s| -> Result<Option<ChannelDescriptor>, ErrorKind> {
            if none_regex.is_match(&s) {
                Ok(None)
            } else {
                Ok(Some(ChannelDescriptor::from_description(&s)?))
            }
        })
        .collect::<Result<Vec<Option<ChannelDescriptor>>, ErrorKind>>())?;

    if descriptors.len() == 0 {
        return Err(CLIError::MissingInput);
    }

    let image = to_dynamic(&descriptors)?;
    if let Some(format) = command.format {
        image.save_with_format(&command.output, format)?;
    } else {
        image.save(&command.output)?;
    }

    log!(quiet, "Done!");

    Ok(())
}

/// Executes a session command.
///
/// Main function starting a session, reading an input folder, and generating
/// the swizzled images.
fn process_session(command: &SessionCommand, quiet: bool) -> Result<(), CLIError> {
    let json = match &command.config {
        Some(path) => std::fs::read_to_string(path),
        _ => {
            let mut buffer = String::new();
            std::io::stdin().read_to_string(&mut buffer)?;
            Ok(buffer)
        }
    }?;

    let mut config: Config = serde_json::from_str(&json)?;

    // Creates a session. This will generate all textures, and save them to disk.
    let session = Session::new()
        .set_output_folder(command.output.to_path_buf())
        .add_targets(&mut config.targets)
        .set_max_threads_nb(command.num_threads);

    // The resolver recursively search for related files in folders. Whenever
    // it matches files together, it save them into a specific structure (an Asset),
    // that the Session will use to generate new textures.
    let mut resolver = GenericAssetReader::new().set_base(config.base);
    for m in config.matchers {
        resolver = resolver.add_matcher(m);
    }

    // Retrieves all assets, generated by the resolver.
    log!(
        quiet,
        format!(
            "Building assets from folder '{}'...",
            command.folder.to_str().unwrap()
        )
    );
    let assets = resolve_assets_dir(&command.folder, &resolver)?;

    // Starts processing all assets, i.e generating the textures and saving
    // them to disk. All errors are reported in a vector.
    log!(
        quiet,
        format!("Running session on {} assets", assets.count())
    );
    let errors = session.run(&assets);
    for e in &errors {
        eprintln!("error found while creating texture: {:?}", e);
    }

    log!(quiet, "Done!");
    Ok(())
}

fn main() {
    let args = Opt::from_args();

    let run = match &args.cmd {
        Command::Manual(manual) => process_manual(&manual, args.quiet),
        Command::Session(session) => process_session(&session, args.quiet),
    };

    if let Err(e) = run {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}
