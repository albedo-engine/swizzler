use image::ImageFormat;
use serde::{de, Deserialize, Deserializer};
use swizzler::session::{GenericTarget, RegexMatcher};

#[derive(Deserialize)]
#[serde(remote = "RegexMatcher")]
struct RegexMatcherDef {
    id: String,
    #[serde(deserialize_with = "de_regexp_from_str")]
    matcher: regex::Regex,
}

#[derive(Deserialize)]
#[serde(remote = "GenericTarget")]
struct GenericTargetDef {
    name: Option<String>,

    #[serde(deserialize_with = "de_image_format_from_str")]
    output_format: image::ImageFormat,

    inputs: Vec<Option<(String, u8)>>,
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(deserialize_with = "de_regexp_from_str")]
    pub base: regex::Regex,

    #[serde(deserialize_with = "de_vec_matcher")]
    pub matchers: Vec<Box<RegexMatcher>>,

    #[serde(deserialize_with = "de_vec_target")]
    pub targets: Vec<GenericTarget>,
}

/// Deserializes a string from a JSON input into a Regex struct.
fn de_regexp_from_str<'de, D>(deserializer: D) -> Result<regex::Regex, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    regex::Regex::new(&s).map_err(de::Error::custom)
}

/// Deserializes a string from a JSON input into an image::ImageFormat.
fn de_image_format_from_str<'de, D>(deserializer: D) -> Result<image::ImageFormat, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let s = String::deserialize(deserializer)?.to_lowercase();
    parse_image_format(&s).map_err(D::Error::custom)
}

/// Deserializes a JSON array into a Vec<Box<RegexMatcher>>.
fn de_vec_matcher<'de, D>(deserializer: D) -> Result<Vec<Box<RegexMatcher>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper(#[serde(with = "RegexMatcherDef")] RegexMatcher);

    let v = Vec::deserialize(deserializer)?;
    Ok(v.into_iter().map(|Wrapper(a)| Box::new(a)).collect())
}

/// Deserializes a JSON array into a Vec<GenericTarget>.
fn de_vec_target<'de, D>(deserializer: D) -> Result<Vec<GenericTarget>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper(#[serde(with = "GenericTargetDef")] GenericTarget);

    let v = Vec::deserialize(deserializer)?;
    Ok(v.into_iter().map(|Wrapper(a)| a).collect())
}

pub fn parse_image_format(input: &str) -> Result<image::ImageFormat, String> {
    match input {
        "png" => Ok(ImageFormat::PNG),
        "jpg" | "jpeg" => Ok(ImageFormat::JPEG),
        "tif" => Ok(ImageFormat::TIFF),
        "tga" => Ok(ImageFormat::TGA),
        "hdr" => Ok(ImageFormat::HDR),
        "bpm" => Ok(ImageFormat::BMP),
        "webp" => Ok(ImageFormat::WEBP),
        "ico" => Ok(ImageFormat::ICO),
        "pnm" => Ok(ImageFormat::PNM),
        _ => Err(format!("unsupported format '{}'", input)),
    }
}
