use std::collections::{
    HashMap
};
use std::path::{
    PathBuf
};
use regex;

pub struct AssetMatcher {
    id: String,
    matcher: regex::Regex
}

impl AssetMatcher {

    pub fn new<T: Into<String>>(id: T, matcher: regex::Regex) -> AssetMatcher {
        AssetMatcher { id: id.into(), matcher }
    }

    fn do_match(&self, filename: &str) -> bool {
        self.matcher.is_match(filename)
    }

}

pub trait AssetReader<'a> {

    type AssetType;

    // TODO: how to take a ref here?
    // If taking a mut ref, it's impossible after to take a immutable ref.

    // Why does resolve needs the lifetime?
    fn resolve(&'a self, files: &Vec<PathBuf>) -> Vec<Self::AssetType>;

}

#[derive(Default)]
pub struct GenericAsset<'a> {
    base: String,
    textures: HashMap<&'a str, PathBuf>
}

impl<'a> GenericAsset<'a> {

    fn new(base: String) -> GenericAsset<'a> {
        GenericAsset { base, ..Default::default() }
    }

    pub fn get_base(&self) -> &str {
        &self.base
    }

    pub fn get_texture_path(&self, id: &str) -> Option<&PathBuf> {
        match self.textures.get(id) {
            Some(path) => Some(path),
            _ => None
        }
    }

}

pub struct GenericAssetReader {
    base: regex::Regex,
    matchers: Vec<AssetMatcher>
}

impl GenericAssetReader {

    pub fn new(
        base: regex::Regex,
        matchers: Vec<AssetMatcher>
    ) -> GenericAssetReader {
        GenericAssetReader {
            base, matchers
        }
    }

}

impl<'a> AssetReader<'a> for GenericAssetReader {

    type AssetType = GenericAsset<'a>;

    fn resolve(&'a self, files: &Vec<PathBuf>) -> Vec<Self::AssetType> {
        let mut result: Vec<Self::AssetType> = Vec::new();

        // TODO: how would it be possible to use a HashMap<&str, usize> here?
        // Obviously this would create mut and immut references at the same
        // time, making the borrow checker un-happy...

        for path in files {
            if let Some(filename) = path.file_name().and_then(|x| x.to_str()) {
                println!("{}", filename);

                let base = self.base.captures(filename).and_then(|v| v.get(1));
                if base.is_none() { continue; }
                let base = base.unwrap().as_str();

                let idx = result.iter()
                    .position(|e| e.base == base)
                    .or_else(|| -> Option<usize> {
                        result.push(Self::AssetType::new(String::from(base)));
                        Some(result.len() - 1)
                    }).unwrap();

                let asset = result.get_mut(idx).unwrap();
                if asset.base == base {
                    for m in &self.matchers {
                        if m.do_match(filename) {
                            // TODO: how to move here instead of clone?
                            asset.textures.insert(&m.id, path.clone());
                        }
                    }
                }
            }
        }

        result

    }

}
