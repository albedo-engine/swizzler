use image::{
    DynamicImage
};
use regex::Regex;
use std::collections::HashMap;

use crate::swizzler::{
    to_rgba
};
use crate::errors::{
    ErrorKind
};

type SwizzleResult = Result<(), ErrorKind>;

type Descriptor<'a> = (&'a str, u8);
pub type ImageSources<'a> = HashMap<String, Textures<'a>>;

#[derive(Default)]
pub struct Textures<'a> {
    albedo: Option<&'a str>,
    roughness: Option<&'a str>,
    metalness: Option<&'a str>
}

impl<'a> Textures<'a> {

    fn new() -> Textures<'a> {
        Default::default()
    }

}

struct UnityMetalRoughnessCommand<'a> {
    metalness: Option<Descriptor<'a>>,
    roughness: Option<Descriptor<'a>>
}

impl<'a> From<UnityMetalRoughnessCommand<'a>> for SwizzleResult {
    fn from(src: UnityMetalRoughnessCommand) -> SwizzleResult {
        to_rgba(r: &Option<ChannelDescriptor>, g: &Option<ChannelDescriptor>, b: &Option<ChannelDescriptor>, a: &Option<ChannelDescriptor>)
    }
}

enum UE4Command {
    MetalRoughness()
}

enum UnityCommand<'a> {
    MetalRoughness(UnityMetalRoughnessCommand<'a>)
}

pub trait TexturesResolver {

    fn resolve_textures<'a>(&self, files: &'a Vec<&'a str>) -> ImageSources<'a> {
        static ROUGH_REGEX: Regex = Regex::new(r"(?i)rough(ness)?").unwrap();
        static METAL_REGEX: Regex = Regex::new(r"(?i)metal(ness)?").unwrap();

        let mut resolved: HashMap<String, Textures> = HashMap::new();

        for f in files {
            let idx = f.rfind("_").unwrap();
            let base = f.split_at(idx).0;
            if resolved.get(base).is_none() {
                resolved.insert(String::from(base), Textures::new());
            }

            let &mut textures = resolved.get_mut(base).unwrap();

            if ROUGH_REGEX.is_match(f) { textures.metalness = Some(f); }
            else if METAL_REGEX.is_match(f) { textures.roughness = Some(f); }
        }

        resolved
    }

}

pub struct DefaultTexturesResolver {}

impl DefaultTexturesResolver {

    pub fn new() -> DefaultTexturesResolver { DefaultTexturesResolver {} }

}

impl TexturesResolver for DefaultTexturesResolver { }

pub trait Swizzler {

    fn swizzle<'a>(
        &self,
        basename: &str,
        textures: &Textures<'a>
    );

}

pub struct UnitySwizzler {}

impl Swizzler for UnitySwizzler {

    fn swizzle<'a>(
        &self,
        out: &mut Vec<Vec<Option<(&'a str, u8)>>>,
        textures: Textures<'a>
    ) {
        match textures {
            Textures { metalness: Some(m), roughness: Some(r), .. } => {
                out.push(vec![ Some((m, 0)), None, None, Some((r, 0)) ]);
            },
            _ => {}
        }
    }

}
