use crate::errors::{
    ErrorKind
};

trait Resolver {

    fn resolve<T>(files: &Vec<T>) where T: AsRef<str>;

}

struct UnityResolver {

}

impl Resolver for UnityResolver {

    fn resolve<T>(files: &Vec<T>)
    where
        T: AsRef<str>
    {

    }

}

#[derive(Default)]
struct Parameters {
    max_nb_threads: u8
}

#[derive(Default)]
pub struct SessionBuilder {
    folders: Vec<std::path::PathBuf>,
    params: Parameters,
}

impl SessionBuilder {

    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(&mut self) -> Result<Session, ErrorKind> {

        for e in &self.folders {
            let mut entries = std::fs::read_dir(e)?
                .map(|res| res.map(|e| e.path()))
                .collect::<Result<Vec<_>, std::io::Error>>()?;
            entries.sort_unstable();
            println!("{:?}", entries)
        }

        Ok(Session {})
    }

    pub fn add_folder<T>(mut self, folder: T) -> Self
    where
        T: Into<std::path::PathBuf>
    {
        self.folders.push(folder.into());
        self
    }

    pub fn add_folders<T, I>(mut self, folders: I) -> Self
    where
        T: Into<std::path::PathBuf>,
        I: IntoIterator<Item = T>
    {
        self.folders.extend(folders.into_iter().map(|e| e.into()));
        self
    }

    pub fn set_max_threads_nb(mut self, count: u8) -> Self {
        self.params.max_nb_threads = count;
        self
    }

}

pub struct Session {

}

impl Session {

    pub fn run() -> () {

    }

}
