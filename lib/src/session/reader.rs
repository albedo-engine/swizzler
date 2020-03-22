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

    /// Returns the identifier associated to this matcher.
    fn get_identifier(&self) -> &Self::Identifier;

}

/// Regex-based file matcher.
///
/// Use this to match images using a regular expression.
#[derive(Clone)]
pub struct RegexMatcher<Id: Eq + Hash = String> {
    pub id: Id,
    pub matcher: regex::Regex
}

impl<Id: Eq + Hash> RegexMatcher<Id> {

    pub fn new(id: Id, matcher: regex::Regex) -> RegexMatcher<Id> {
        RegexMatcher { id, matcher }
    }

}

impl<Id: Eq + Hash> FileMatch for RegexMatcher<Id> {

    type Identifier = Id;

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

pub struct GenericAsset<'a, Id: Eq + Hash + 'a = String> {
    base: String,
    textures: HashMap<&'a Id, PathBuf>
}

impl<'a, Id: Eq + Hash> GenericAsset<'a, Id> {

    fn new(base: String) -> GenericAsset<'a, Id> {
        GenericAsset { base, textures: HashMap::new() }
    }

    pub fn get_texture_path(&self, id: &Id) -> Option<&PathBuf> {
        self.textures.get(id)
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
pub struct GenericAssetReader<I: Eq + Hash = String> {
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

    /// Sets the regex used to extract the name of each asset.
    ///
    /// This regex is extremly important as it's the one used to group images
    /// together under the same asset.
    ///
    /// # Example
    ///
    /// ```sh
    /// $ ls
    /// hero_metalness.png
    /// hero_roughness.png
    /// hero_ao.png
    /// enemy_metalness.png
    /// enemy_roughness.png
    /// enemy_ao.png
    /// ```
    ///
    /// Here we want the base name to be everything up to the last underscore,
    /// we can then use this regexp to extract the base:
    ///
    /// ```rust
    /// set_base(Regex::new(r"(.*)_.*").unwrap())
    /// ```
    pub fn set_base(mut self, base: regex::Regex) -> Self {
        // TODO: check that base as at least one capture.
        self.base = base;
        self
    }

    /// Adds a matcher to this reader.println
    ///
    /// All matchers will be run on all files in order to determine their type.
    ///
    /// # Example
    ///
    /// ```rust
    ///
    /// ```
    pub fn add_matcher(mut self, matcher: Box<dyn FileMatch<Identifier=I>>) -> Self {
        self.matchers.push(matcher);
        self
    }

    pub fn set_matchers(mut self, matchers: Vec<Box<dyn FileMatch<Identifier=I>>>) -> Self {
        self.matchers = matchers;
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

/// List of assets resolved relative to a given root folder.
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

/// Resolves an assets directory.
///
/// This function generates an [`AssetBundle`] that you can process using
/// a [`Session`].
///
/// The function uses the ```resolver``` argument to process recursively the
/// folder pointed by the ```dir``` argument.
///
/// # Example
///
/// ```rust
/// let resolver = GenericAssetReader::new();
///
/// // Resolves all assets in the current directory, using the `GenericAssetReader`.
/// let assets = resolve_assets_dir(
///     std::path::Path::new("./"),
///     &resolver
/// );
/// ```
///
/// ```dir``
pub fn resolve_assets_dir<'a, A: Asset, Resolver: AssetReader<'a, A>>(
    dir: &Path,
    resolver: &'a Resolver
) -> Result<AssetBundle<A>, ErrorKind> {
    let mut bundle = AssetBundle { root: dir.to_path_buf(), assets: Vec::new() };
    resolve_dir_rec(dir, &mut bundle.assets, resolver)?;
    bundle.assets.retain(|e| !e.empty());
    Ok(bundle)
}

/// Recursive body of `resolve_assets_dir`.
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
    // Retains only files, so that the resolver doesn't take care of discarding
    // paths pointing to directory.
    files.retain(|p| p.is_file());
    out.append(&mut resolver.resolve(&files));
    Ok(())
}
