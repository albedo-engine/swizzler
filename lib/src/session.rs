use image::{
    GenericImageView
};
use std::thread;

use crate::errors::{
    ErrorKind
};

use crate::swizzler::{
    ChannelDescriptor,
    to_dynamic
};

use std::collections::HashMap;

pub fn unityresolve<'a>(files: &Vec<&'a std::path::PathBuf>) -> Option<Command<'a>> {
    let mut metalness: Option<ChannelFileDesc<'a>> = None;
    let mut roughness: Option<ChannelFileDesc<'a>> = None;

    for fpath in files {
        let str = fpath.to_str().unwrap();
        if str.rfind("_metalness").is_some() {
            metalness = Some(ChannelFileDesc {
                channel: 0,
                file_path: fpath
            })
        } else if str.rfind("_roughness").is_some() {
            roughness = Some(ChannelFileDesc {
                channel: 0,
                file_path: fpath
            });
        }
    }

    if metalness.is_some() || roughness.is_some() {
        return Some(Command {
            name: std::path::PathBuf::from("testounet"),
            desc: vec![
                metalness,
                None,
                None,
                roughness
            ]
        });
    }

    None

}

pub struct ChannelFileDesc<'a> {
    channel: u8,
    file_path: &'a std::path::PathBuf,
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

impl SessionBuilder {

    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(&mut self) -> Result<(), ErrorKind> {

        for e in &self.folders {
            let mut entries = std::fs::read_dir(e)?
                .map(|res| res.map(|e| e.path()))
                .collect::<Result<Vec<_>, std::io::Error>>()?;
            entries.sort_unstable();
            println!("{:?}", entries);

            let mut map: HashMap<String, Vec<&std::path::PathBuf>> = HashMap::new();

            for element in &entries {
                if let Some(filename) = element.file_name() {
                    if let Some(name) = filename.to_str() {
                        let idx = name.rfind("_").unwrap();
                        let base = name.split_at(idx).0;
                        if map.get(base).is_none() {
                            map.insert(String::from(base), Vec::new());
                        }
                        map.get_mut(base).unwrap().push(element);
                    }
                }
            }

            let mut cmds: Vec<Command> = Vec::new();

            for (name, files) in &map {
                println!("{}", name);
                println!("{:?}", files);
                if let Some(v) = unityresolve(&files) {
                    cmds.push(v);
                }
            }

            use std::time::Instant;
            let now = Instant::now();

            const nthreads: usize = 3;

            // let mut handles: Vec<std::thread::JoinHandle<()>> =
               // Vec::with_capacity(nthreads);

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

                    scope.spawn(move |_| {
                        println!("new thread");
                        for cmd in slice {
                            let val: Vec<Option<ChannelDescriptor>> = cmd.desc.iter().map(
                                |x| -> Result<Option<ChannelDescriptor>, ErrorKind>  {
                                    match &x {
                                        Some(val) => Ok(Some(ChannelDescriptor::from_path(val.file_path, val.channel)?)),
                                        _ => Ok(None)
                                    }
                                }
                            ).collect::<Result<Vec<Option<ChannelDescriptor>>, ErrorKind>>().unwrap();

                            let img = to_dynamic(&val).unwrap();
                        }
                    });
                }
            });

            let elapsed = now.elapsed();
            println!("Elapsed: {:.2}", elapsed.as_millis());

            // let img = to_dynamic(&val)?;
            // println!("{}", img.dimensions().0);
        }

        Ok(())
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
