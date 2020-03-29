mod reader;
pub use reader::{
    resolve_assets_dir, Asset, AssetBundle, AssetReader, FileMatch, GenericAsset,
    GenericAssetReader, RegexMatcher,
};

mod writer;
pub use writer::{GenericTarget, Target};

use std::path::{PathBuf};
use crate::errors::ErrorKind;

struct Parameters {
    max_nb_threads: usize,
}

impl Parameters {
    fn new() -> Parameters {
        Parameters {
            max_nb_threads: num_cpus::get(),
        }
    }
}

pub struct Session<AssetType: Asset + Sync, T: Target<AssetType> + Sync> {
    output_folder: PathBuf,

    targets: Vec<T>,

    parameters: Parameters,

    _phantom: std::marker::PhantomData<AssetType>,
}

impl<AssetType: Asset + Sync, T: Target<AssetType> + Sync> Session<AssetType, T> {
    pub fn new() -> Session<AssetType, T> {
        Session {
            output_folder: PathBuf::from("./__swizzler_build"),
            targets: Vec::new(),
            parameters: Parameters::new(),
            _phantom: std::marker::PhantomData {},
        }
    }

    pub fn set_output_folder(mut self, folder: PathBuf) -> Self {
        self.output_folder = folder;
        self
    }

    pub fn run(&self, bundle: &AssetBundle<AssetType>) -> Vec<ErrorKind> {
        // TODO: clean up the function
        // TODO: remove temporary allocations of Vec

        let errors = std::sync::Mutex::new(Vec::new());

        let write_func = |target: &T, asset: &AssetType| -> Result<(), ErrorKind> {
            let img = target.generate(asset)?;
            let mut fullpath = self.output_folder.to_path_buf();
            if let Some(p) = asset.get_folder() {
                fullpath.push(p.strip_prefix(bundle.get_root()).unwrap());
            }
            fullpath.push(target.get_filename(asset));

            // Creates directory if doesn't exist.
            std::fs::create_dir_all(fullpath.parent().unwrap())?;

            img.save_with_format(&fullpath, target.get_format())?;
            Ok(())
        };

        let worker_func = |assets: &[AssetType]| {
            for asset in assets {
                for target in &self.targets {
                    if let Err(e) = write_func(target, asset) {
                        let mut data = errors.lock().unwrap();
                        data.push(e);
                    }
                }
            }
        };

        let assets = bundle.get_assets();
        let nthreads = std::cmp::min(assets.len() / 2, self.parameters.max_nb_threads);
        let slice_size: usize = assets.len() / nthreads;

        crossbeam::scope(|scope| {
            for i in 0..nthreads {
                let start = i * slice_size;
                let slice = if i < nthreads - 1 {
                    &assets[start..(start + slice_size)]
                } else {
                    &assets[start..]
                };

                scope.spawn(move |_| (worker_func(slice)));
            }
        }).unwrap();

        errors.into_inner().unwrap()
    }

    pub fn add_target(mut self, target: T) -> Self {
        self.targets.push(target);
        self
    }

    pub fn add_targets(mut self, targets: &mut Vec<T>) -> Self {
        self.targets.append(targets);
        self
    }

    pub fn set_max_threads_nb(mut self, count: Option<usize>) -> Self {
        self.parameters.max_nb_threads = count.or(Some(num_cpus::get())).unwrap();
        self
    }
}
