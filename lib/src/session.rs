use std::path::{Path, PathBuf};

use crate::asset::{
    AssetReader, GenericWriter, GenericAssetReader, GenericAsset,
    GenericTarget, Target
};

use crate::errors::{
    ErrorKind
};

struct Parameters {
    max_nb_threads: usize
}

impl Parameters {

    fn new() -> Parameters {
        Parameters {
            max_nb_threads: num_cpus::get()
        }
    }

}

pub struct Session<'a> {

    assets: Vec<GenericAsset<'a>>,

    input_folder: std::path::PathBuf,

    output_folder: std::path::PathBuf,

    parameters: Parameters

}

impl<'a> Session<'a> {

    pub fn new() -> Session<'a> {
        Session {
            output_folder: std::path::PathBuf::from("./__swizzler_build"),
            input_folder: std::path::PathBuf::new(),
            parameters: Parameters::new(),
            assets: Vec::new()
        }
    }

    pub fn set_output_folder(mut self, folder: std::path::PathBuf) -> Self {
        self.output_folder = folder;
        self
    }

    pub fn read(&mut self, resolver: &'a GenericAssetReader) -> Result<(), ErrorKind> {
        resolve_dir(&self.input_folder, &mut self.assets, resolver)?;
        self.assets.retain(|e| !e.empty());
        Ok(())
    }

    pub fn run(&self, swizzler: &GenericWriter) -> Vec<ErrorKind> {
        // TODO: clean up the function
        // TODO: remove temporary allocations of Vec

        let errors = std::sync::Mutex::new(Vec::new());

        let write_func = |target: &GenericTarget, asset: &GenericAsset| ->
            Result<(), ErrorKind> {
            let img = target.generate(asset)?;
            let mut fullpath = self.output_folder.to_path_buf();
            if let Some(p) = asset.get_folder() {
                fullpath.push(p.strip_prefix(&self.input_folder).unwrap());
            }
            fullpath.push(target.get_filename(asset));

            // Creates directory if doesn't exist.
            std::fs::create_dir_all(fullpath.parent().unwrap())?;

            img.save_with_format(&fullpath, target.output_format)?;
            Ok(())
        };

        let worker_func = |assets: &[ GenericAsset ]| {
            for asset in assets {
                for target in &swizzler.targets {
                    if let Err(e) = write_func(target, asset) {
                        let mut data = errors.lock().unwrap();
                        data.push(e);
                    }
                }
            }
        };

        let nthreads = std::cmp::min(
            self.assets.len() / 2,
            self.parameters.max_nb_threads
        );

        let slice_size: usize = self.assets.len() / nthreads;

        crossbeam::scope(|scope| {
            for i in 0..nthreads {
                let start = i * slice_size;
                let slice = if i < nthreads - 1 {
                    &self.assets[start..(start + slice_size)]
                } else {
                    &self.assets[start..]
                };

                scope.spawn(move|_| (worker_func(slice)));
            }
        });

        errors.into_inner().unwrap()
    }

    pub fn set_input_folder<U>(mut self, folder: U) -> Self
    where
        U: Into<std::path::PathBuf>
    {
        self.input_folder = folder.into();
        self
    }

    pub fn set_max_threads_nb(mut self, count: usize) -> Self {
        self.parameters.max_nb_threads = count;
        self
    }

}

// one possible implementation of walking a directory only visiting files
fn resolve_dir<'a>(
    dir: &Path,
    out: &mut Vec<GenericAsset<'a>>,
    resolver: &'a GenericAssetReader,
) -> std::io::Result<()> {
    let mut files = std::fs::read_dir(dir)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    for path in &files {
        if path.is_dir() { resolve_dir(path, out, resolver)?; }
    }
    files.retain(|p| p.is_file());

    out.append(&mut resolver.resolve(&files));
    Ok(())
}
