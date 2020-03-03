use image::{
    GenericImageView
};

use crate::errors::{
    ErrorKind
};

use crate::swizzler::{
    ChannelDescriptor,
    to_dynamic
};

use crate::resolver::{
    DefaultResolver,
    ImageSources,
    Swizzler,
    Resolver
};

#[derive(Default)]
struct Parameters {
    max_nb_threads: u8
}

pub struct SessionBuilder<T: Resolver> {
    folders: Vec<std::path::PathBuf>,
    errors: Vec<ErrorKind>,
    resolver: Option<T>,
    params: Parameters
}

pub struct Command<'a> {

    name: std::path::PathBuf,

    desc: Vec<Option<ChannelFileDesc<'a>>>,

}

impl<'a> Command<'a> {

    fn new(name: std::path::PathBuf) -> Command<'a> {
        Command {
            name,
            desc: Vec::new()
        }
    }

}

impl<T: Resolver> SessionBuilder<T> {

    pub fn new() -> Self {
        SessionBuilder {
            folders: Default::default(),
            errors: Default::default(),
            resolver: None,
            params: Default::default()
        }
    }

    pub fn build<'a>(mut self) -> Result<Session<'a>, ErrorKind> {
        self.errors.clear();

        let mut files: Vec<String> = Vec::new();

        for e in &self.folders {
            let mut entries = std::fs::read_dir(e)?
                .map(|res| res.map(|e| e.path()))
                .collect::<Result<Vec<_>, std::io::Error>>()?;
            files.append(&mut entries);
        }
        files.sort_unstable();

        let sources = match &self.resolver {
            Some(r) => r.resolve(&files),
            _ => {
                let default_resolver = DefaultResolver::new();
                default_resolver.resolve(&files)
            }
        };

        Ok(Session {
            files,
            sources
        })
    }

    pub fn add_folder<U>(mut self, folder: U) -> Self
    where
        U: Into<std::path::PathBuf>
    {
        self.folders.push(folder.into());
        self
    }

    pub fn add_folders<U, I>(mut self, folders: U) -> Self
    where
    U: Into<std::path::PathBuf>,
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

pub struct Session<'a> {

    files: Vec<std::path::PathBuf>,

    sources: ImageSources<'a>

}

impl<'a> Session<'a> {

    pub fn run(&self, swizzler: &dyn Swizzler) -> Vec<ErrorKind> {
        // TODO: clean up the function
        // TODO: remove temporary allocations of Vec

        let commands: Vec<(&str, u8)> = Vec::new();

        for (name, files) in &self.sources {
            if let Some(v) = swizzler.swizzle(&files) {
                cmds.push(v);
            }
        }

        let errors = std::sync::Mutex::new(Vec::new());

        let worker_func = |cmds: &[ Command ]| {
            for cmd in cmds {
                if let Err(e) = process_command(cmd) {
                    let mut data = errors.lock().unwrap();
                    data.push(e);
                }
            }
        };

        const nthreads: usize = 3;

        let slice_size: usize = cmds.len() / nthreads;
        println!("{}", cmds.len());

        crossbeam::scope(|scope| {
            for i in 0..nthreads {
                let start = i * slice_size;
                let slice = if i < nthreads - 1 {
                    &cmds[start..(start + slice_size)]
                } else {
                    &cmds[start..]
                };

                scope.spawn(move|_| (worker_func(slice)));
            }
        });

        errors.into_inner().unwrap()
    }

}

fn process_command(cmd: &Command) -> Result<(), ErrorKind> {
    let val: Vec<Option<ChannelDescriptor>> = cmd.desc.iter().map(|x| {
        match &x {
            Some(val) => Ok(Some(ChannelDescriptor::from_path(val.file_path, val.channel)?)),
            _ => Ok(None)
        }
    }
    ).collect::<Result<Vec<Option<ChannelDescriptor>>, ErrorKind>>()?;

    let img = to_dynamic(&val)?;
    img.save("").map_err(|e| e.into())
}
