mod reader;
pub use reader::{
    AssetReader,
    AssetMatcher,
    GenericAsset,
    GenericAssetReader
};

mod writer;
pub use writer::{
    GenericTarget,
    GenericWriter,
    Target
};

/* pub struct DefaultAssetResolver;
impl AssetResolver for DefaultAssetResolver {

    type Output = DefaultAsset;

    fn resolve<'a>(&self, files: Vec<std::path::PathBuf>) -> Vec<Self::Output> {
        static ROUGH_REGEX: Regex = Regex::new(r"(?i)rough(ness)?").unwrap();
        static METAL_REGEX: Regex = Regex::new(r"(?i)metal(ness)?").unwrap();

        let resolved: HashMap<&str, &DefaultAsset> = HashMap::new();
        let result: Vec<DefaultAsset> = Vec::new();

        for path in files {
            if let Some(filename) = path.file_name().and_then(|x| x.to_str()) {
                let idx = filename.rfind("_").unwrap();
                let base = filename.split_at(idx).0;
                if resolved.get(base).is_none() {
                    let asset = DefaultAsset::new(String::from(base));
                    result.push(asset);
                    resolved.insert(&asset.base, &asset);
                }
                let &mut asset = resolved.get_mut(&base).unwrap();
                if ROUGH_REGEX.is_match(filename) {
                    asset.metalness = Some(path);
                } else if METAL_REGEX.is_match(filename) {
                    asset.roughness = Some(path);
                }
            }
        }

        result
    }

}*/
