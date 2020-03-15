use image::{
    GenericImageView
};

use crate::asset::{
    AssetReader, GenericWriter, GenericAssetReader, GenericAsset,
    GenericTarget, Target
};

use crate::errors::{
    ErrorKind
};

#[derive(Default)]
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

#[derive(Default)]
pub struct SessionBuilder {
    folders: Vec<std::path::PathBuf>,
    params: Parameters
}

impl SessionBuilder {

    pub fn new() -> Self {
        SessionBuilder {
            params: Parameters::new(),
            ..Default::default()
        }
    }

    pub fn build<'a>(
        self,
        resolver: &'a GenericAssetReader
    ) -> Result<Session<'a>, ErrorKind> {
        let mut assets: Vec<GenericAsset<'a>> = Vec::new();

        for e in &self.folders {
            let mut entries = std::fs::read_dir(e)?
                .map(|res| res.map(|e| e.path()))
                .collect::<Result<Vec<_>, std::io::Error>>()?;
            entries.sort_unstable();
            assets.append(&mut resolver.resolve(&entries));
        }

        Ok(Session::new(assets, self.params))
    }

    pub fn add_folder<U>(mut self, folder: U) -> Self
    where
        U: Into<std::path::PathBuf>
    {
        self.folders.push(folder.into());
        self
    }

    pub fn add_folders<U, I>(mut self, folders: I) -> Self
    where
    U: Into<std::path::PathBuf>,
    I: IntoIterator<Item = U>
    {
        self.folders.extend(folders.into_iter().map(|e| e.into()));
        self
    }

    pub fn set_max_threads_nb(mut self, count: usize) -> Self {
        self.params.max_nb_threads = count;
        self
    }

}

pub struct Session<'a> {

    assets: Vec<GenericAsset<'a>>,

    output_folder: std::path::PathBuf,

    parameters: Parameters

}

impl<'a> Session<'a> {

    fn new(assets: Vec<GenericAsset<'a>>, parameters: Parameters) -> Session {
        Session {
            assets,
            parameters,
            output_folder: std::path::PathBuf::from("./__swizzler_build")
        }
    }

    pub fn set_output_folder(mut self, folder: std::path::PathBuf) -> Self {
        self.output_folder = folder;
        self
    }

    pub fn run(&self, swizzler: &GenericWriter) -> Vec<ErrorKind> {
        // TODO: clean up the function
        // TODO: remove temporary allocations of Vec

        let output_folder = &self.output_folder;

        let errors = std::sync::Mutex::new(Vec::new());

        let write_func = |target: &GenericTarget, asset: &GenericAsset| ->
            Result<(), ErrorKind> {
            let img = target.generate(asset)?;
            let mut p = output_folder.to_path_buf();
            p.push(target.get_filename(asset));
            img.save_with_format(&p, target.output_format)?;
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

}
