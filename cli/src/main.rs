use structopt::StructOpt;
use serde::{
    Deserialize,
    Deserializer,
    de
};
use image::ImageFormat;

use swizzler::{
    errors::ErrorKind,
    ChannelDescriptor,
    to_dynamic
};
use swizzler::session::{
    GenericAssetReader,
    GenericTarget,
    RegexMatcher,
    FileMatch,
    Session,
    resolve_assets_dir
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
#[structopt(rename_all = "kebab-case")]
struct SessionCommand {

    #[structopt(long = "folder", short, parse(from_os_str))]
    folder: std::path::PathBuf,

    #[structopt(
        long = "output",
        short,
        parse(from_os_str),
        default_value = "./__swizzler_build"
    )]
    output: std::path::PathBuf

}

#[derive(StructOpt)]
enum Command {

    Manual(ManualCommand),

    Session(SessionCommand)

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

#[derive(Deserialize)]
#[serde(remote = "RegexMatcher")]
struct RegexMatcherDef {
    id: String,
    #[serde(deserialize_with = "de_regexp_from_str")]
    matcher: regex::Regex
}

#[derive(Deserialize)]
#[serde(remote = "GenericTarget")]
struct GenericTargetDef {
    name: Option<String>,

    #[serde(deserialize_with = "de_image_format_from_str")]
    output_format: image::ImageFormat,

    inputs: Vec<Option<(String, u8)>>
}

#[derive(Deserialize)]
struct Config {

    #[serde(deserialize_with = "de_regexp_from_str")]
    base: regex::Regex,

    #[serde(deserialize_with = "de_vec_matcher")]
    matchers: Vec<Box<RegexMatcher>>,

    #[serde(deserialize_with = "de_vec_target")]
    targets: Vec<GenericTarget>

}

fn de_regexp_from_str<'de, D>(deserializer: D) -> Result<regex::Regex, D::Error>
    where D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    regex::Regex::new(&s).map_err(de::Error::custom)
}

fn de_image_format_from_str<'de, D>(deserializer: D) -> Result<image::ImageFormat, D::Error>
    where D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?.to_lowercase();
    match s.as_str() {
        "png" => Ok(ImageFormat::PNG),
        "jpg" => Ok(ImageFormat::JPEG),
        "tif" => Ok(ImageFormat::TIFF),
        "tga" => Ok(ImageFormat::TGA),
        "hdr" => Ok(ImageFormat::HDR),
        "gif" => Ok(ImageFormat::GIF),
        "bpm" => Ok(ImageFormat::BMP),
        "webp" => Ok(ImageFormat::WEBP),
        "ico" => Ok(ImageFormat::ICO),
        "pnm" => Ok(ImageFormat::PNM),
        _ => Ok(ImageFormat::PNG)
    }
}

fn de_vec_matcher<'de, D>(deserializer: D) -> Result<Vec<Box<RegexMatcher>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper(#[serde(with = "RegexMatcherDef")] RegexMatcher);

    let v = Vec::deserialize(deserializer)?;
    Ok(v.into_iter().map(|Wrapper(a)| Box::new(a)).collect())
}

fn de_vec_target<'de, D>(deserializer: D) -> Result<Vec<GenericTarget>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper(#[serde(with = "GenericTargetDef")] GenericTarget);

    let v = Vec::deserialize(deserializer)?;
    Ok(v.into_iter().map(|Wrapper(a)| a).collect())
}

/// Executes a manual command.
///
/// A manual command takes up to four input images, and swizzle their channels
/// into a new image. This allows user to swizzle anything using this CLI.
fn process_manual(command: &ManualCommand) -> Result<(), ErrorKind> {
    // Converts inputs into channel descriptors, that the Swizzler library
    // can use to generate the image.
    let descriptors: Vec<Option<ChannelDescriptor>> =
        (command.descriptors
            .iter()
            .map(|s| -> Result<Option<ChannelDescriptor>, ErrorKind> {
                Ok(Some(ChannelDescriptor::from_description(&s)?))
            })
            .collect::<Result<Vec<Option<ChannelDescriptor>>, ErrorKind>>()
        )?;

    let image = to_dynamic(&descriptors)?;
    image.save(&command.output)?;
    Ok(())
}

fn process_session(command: &SessionCommand) -> Result<(), ErrorKind> {

    let json = r#"{
        "base": "(.*)_.*",
        "matchers": [
            { "id": "metalness", "matcher": "(?i)metal(ness)?" },
            { "id": "roughness", "matcher": "(?i)rough(ness)?" }
        ],
        "targets": [
          {
            "name": "-metalness-roughness.png",
            "output_format": "png",
            "inputs": [
                [ "metalness", 0 ],
                null,
                null,
                [ "roughness", 0 ]
            ]
          }
        ]
      }"#;

    let mut config: Config = serde_json::from_str(json).unwrap();

    println!("{}", config.base.as_str());

    let session = Session::new()
        .set_output_folder(command.output.to_path_buf())
        .add_targets(&mut config.targets);

    let mut resolver = GenericAssetReader::new().set_base(config.base);
    for m in config.matchers {
        resolver = resolver.add_matcher(m);
    }

    let assets = resolve_assets_dir(&command.folder, &resolver)?;

    /* let generic_reader = GenericAssetReader::new()
        .add_matcher(
            Box::new(RegexMatcher::new(String::from("metalness"), Regex::new(r"(?i)metal(ness)?").unwrap()))
        )
        .add_matcher(
            Box::new(RegexMatcher::new(String::from("roughness"), Regex::new(r"(?i)rough(ness)?").unwrap()))
        );

    let assets = resolve_assets_dir(std::path::Path::new("./"), &generic_reader)?;

    let target = GenericTarget::new(vec! [
        Some((String::from("metalness"), 0)),
        None,
        None,
        Some((String::from("roughness"), 0))
    ]).set_name(String::from("-metalroughness.png"));

    let session = Session::new()
        .set_output_folder(command.output.to_path_buf())
        .add_target(target); */

    let errors = session.run(&assets);

    println!("{}", errors.len());
    for e in &errors {
        eprintln!("Error found: {:?}", e);
    }
    Ok(())
}

fn main() -> Result<(), ErrorKind> {
    let args = Opt::from_args();

    match &args.cmd {
        Command::Manual(manual) => {
            process_manual(&manual)
        },
        Command::Session(session) => {
            process_session(&session)
        },
        _ => Ok(())
    }

}
