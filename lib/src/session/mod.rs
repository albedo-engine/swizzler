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
