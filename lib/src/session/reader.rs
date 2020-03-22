use std::cmp::Eq;
use std::hash::Hash;
use std::collections::{
    HashMap
};
use std::path::{
    PathBuf, Path
};
use regex;

use crate::errors::{
    ErrorKind
};

/// Generalized file name matcher.
pub trait FileMatch {

    type Identifier;

    /// Returns `true` if ```filename``` matches the expected format. Returns
    /// `false` otherwise.
    fn do_match(&self, filename: &str) -> bool;

    fn get_identifier(&self) -> &Self::Identifier;

}

pub struct RegexMatcher<Identifier: Eq + Hash> {
    id: Identifier,
    matcher: regex::Regex
}

impl<Identifier: Eq + Hash> RegexMatcher<Identifier> {

    pub fn new(id: Identifier, matcher: regex::Regex) -> RegexMatcher<Identifier> {
        RegexMatcher { id, matcher }
    }

}

impl<Identifier: Eq + Hash> FileMatch for RegexMatcher<Identifier> {

    type Identifier = Identifier;

    fn do_match(&self, filename: &str) -> bool {
        self.matcher.is_match(filename)
    }

    fn get_identifier(&self) -> &Self::Identifier {
        &self.id
    }

}

/// Generalized asset.
///
/// Assets should contain files that are related one to another. Those files
/// will later be swizzled together into new textures.
///
/// Assets should be created by an [`AssetReader`].
pub trait Asset {

    /// Returns `true` if this assets has no registered files.
    /// `false` otherwise.
    fn empty(&self) -> bool;

    /// Returns the base name of the asset.
    fn get_base(&self) -> &str;

    /// Returns the path to the parent folder, if any.
    fn get_folder(&self) -> Option<&Path>;

}

pub struct GenericAsset<'a, Identifier: Eq + Hash + 'a> {
    base: String,
    textures: HashMap<&'a Identifier, PathBuf>
}

impl<'a, Identifier: Eq + Hash> GenericAsset<'a, Identifier> {

    fn new(base: String) -> GenericAsset<'a, Identifier> {
        GenericAsset { base, textures: HashMap::new() }
    }

    pub fn get_texture_path(&self, id: &Identifier) -> Option<&PathBuf> {
        match self.textures.get(id) {
            Some(path) => Some(path),
            _ => None
        }
    }

}

impl<'a, Identifier: Eq + Hash> Asset for GenericAsset<'a, Identifier> {

    fn empty(&self) -> bool {
        self.textures.len() == 0
    }

    fn get_base(&self) -> &str {
        &self.base
    }

    fn get_folder(&self) -> Option<&Path> {
        match self.textures.values().next() {
            Some(p) => p.parent(),
            None => None
        }
    }

}

/// Generalized assets reader.
///
/// Implement this trait to provide custom logic for file reading.
pub trait AssetReader<'a, A: Asset> {

    /// Given a list of files, produce a vector of assets.
    /// Assets should contain files that are related one to another.
    fn resolve(&'a self, files: &Vec<PathBuf>) -> Vec<A>;

}

/// Generic assets reader.
///
/// Uses ```RegexMatcher``` to match assets together into their own
/// ```GenericAsset``` container.
pub struct GenericAssetReader<I: Eq + Hash> {
    base: regex::Regex,
    matchers: Vec<Box<dyn FileMatch<Identifier=I>>>
}

impl<I: Eq + Hash> GenericAssetReader<I> {

    pub fn new() -> GenericAssetReader<I> {
        GenericAssetReader {
            base: regex::Regex::new(r"(.*)_.*").unwrap(),
            matchers: Vec::new()
        }
    }

    pub fn set_base(mut self, base: regex::Regex) -> Self {
        self.base = base;
        self
    }

    pub fn add_matcher(mut self, matcher: Box<dyn FileMatch<Identifier=I>>) -> Self {
        self.matchers.push(matcher);
        self
    }

}

impl<'a, I: Eq + Hash + 'a> AssetReader<'a, GenericAsset<'a, I>> for GenericAssetReader<I> {

    fn resolve(&'a self, files: &Vec<PathBuf>) -> Vec<GenericAsset<'a, I>> {
        let mut result: Vec<GenericAsset<'a, I>> = Vec::new();

        // TODO: how would it be possible to use a HashMap<&str, usize> here?
        // Obviously this would create mut and immut references at the same
        // time, making the borrow checker un-happy...

        for path in files {
            if let Some(filename) = path.file_name().and_then(|x| x.to_str()) {
                let base = self.base.captures(filename).and_then(|v| v.get(1));
                if base.is_none() { continue; }

                let base = base.unwrap().as_str();

                let idx = result.iter()
                    .position(|e| e.base == base)
                    .or_else(|| -> Option<usize> {
                        let asset = GenericAsset::new(String::from(base));
                        result.push(asset);
                        Some(result.len() - 1)
                    }).unwrap();

                let asset = result.get_mut(idx).unwrap();
                if asset.base == base {
                    for m in &self.matchers {
                        if m.do_match(filename) {
                            // TODO: how to move here instead of clone?
                            asset.textures.insert(&m.get_identifier(), path.clone());
                        }
                    }
                }
            }
        }

        result
    }

}

pub struct AssetBundle<A: Asset> {
    root: PathBuf,
    assets: Vec<A>
}

impl<A: Asset> AssetBundle<A> {

    pub fn get_root(&self) -> &Path {
        &self.root
    }

    pub fn get_assets(&self) -> &Vec<A> {
        &self.assets
    }

}

pub fn resolve_assets_dir<'a, A: Asset, Resolver: AssetReader<'a, A>>(
    dir: &Path,
    resolver: &'a Resolver
) -> Result<AssetBundle<A>, ErrorKind> {
    let mut bundle = AssetBundle { root: dir.to_path_buf(), assets: Vec::new() };
    resolve_dir_rec(dir, &mut bundle.assets, resolver)?;
    bundle.assets.retain(|e| !e.empty());
    Ok(bundle)
}

// one possible implementation of walking a directory only visiting files
fn resolve_dir_rec<'a, A: Asset, Resolver: AssetReader<'a, A>>(
    curr_dir: &Path,
    out: &mut Vec<A>,
    resolver: &'a Resolver,
) -> std::io::Result<()> {
    let mut files = std::fs::read_dir(curr_dir)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    for path in &files {
        if path.is_dir() { resolve_dir_rec(path, out, resolver)?; }
    }
    files.retain(|p| p.is_file());

    out.append(&mut resolver.resolve(&files));
    Ok(())
}
