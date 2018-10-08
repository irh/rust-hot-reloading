extern crate libloading as lib;
extern crate notify;

use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::{fs, time};

use notify::op::*;
use notify::{raw_watcher, RawEvent, RecursiveMode, Watcher};

pub struct HotReloadLib {
    original_lib_path_string: String,
    original_lib_path: PathBuf,
    loaded_path: PathBuf,
    library: Option<lib::Library>,
    watch_event_receiver: Receiver<RawEvent>,
    _watcher: notify::RecommendedWatcher,
}

impl HotReloadLib {
    pub fn new(folder: &str, lib_name: &str) -> Self {
        let lib_path_string = {
            let prefix = "lib";
            let extension = "dylib";
            format!("{}/{}{}.{}", folder, prefix, lib_name, extension)
        };
        let lib_path = Path::new(&lib_path_string).canonicalize().unwrap();
        let (tx, rx) = channel();
        let (library, loaded_path) = copy_and_load_library(&lib_path_string);
        let mut watcher = raw_watcher(tx).unwrap();
        watcher.watch(folder, RecursiveMode::NonRecursive).unwrap();

        HotReloadLib {
            original_lib_path_string: lib_path_string,
            original_lib_path: lib_path,
            loaded_path: loaded_path,
            library: Some(library),
            watch_event_receiver: rx,
            _watcher: watcher,
        }
    }

    pub fn load_symbol<Signature>(&self, symbol_name: &str) -> lib::Symbol<Signature> {
        match self.library {
            Some(ref x) => unsafe {
                x.get(symbol_name.as_bytes())
                    .expect(format!("Failed to find symbol '{:?}'", symbol_name).as_str())
            },
            None => panic!(),
        }
    }

    pub fn update(&mut self) {
        for event in self.watch_event_receiver.try_iter() {
            if let RawEvent {
                path: Some(path),
                op: Ok(op),
                cookie: _,
            } = event
            {
                if path.as_path() == self.original_lib_path {
                    if op == CREATE | REMOVE {
                        self.library = None; // Work around library not reloading
                        fs::remove_file(&self.loaded_path).unwrap();
                        let (library, path) = copy_and_load_library(&self.original_lib_path_string);
                        self.library = Some(library);
                        self.loaded_path = path;
                    }
                }
            }
        }
    }
}

impl Drop for HotReloadLib {
    fn drop(&mut self) {
        fs::remove_file(&self.loaded_path).unwrap();
    }
}

fn copy_and_load_library(lib_path: &String) -> (lib::Library, PathBuf) {
    let unique_name = {
        let timestamp = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let index = lib_path.rfind('.').unwrap();
        let (before, after) = lib_path.split_at(index);
        format!("{}-{}{}", before, timestamp, after)
    };
    fs::copy(&lib_path, &unique_name).expect("Failed to copy lib to unique path");
    let unique_lib_path = Path::new(&unique_name).canonicalize().unwrap();
    (
        lib::Library::new(unique_lib_path.as_os_str())
            .expect(format!("Failed to load library '{:?}'", unique_lib_path).as_str()),
        unique_lib_path,
    )
}
