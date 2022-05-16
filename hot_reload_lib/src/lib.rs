extern crate libloading as lib;
extern crate notify;

use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::{fs, time};

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

            #[cfg(target_os = "macos")]
            let extension = "dylib";
            #[cfg(target_os = "linux")]
            let extension = "so";

            format!("{}/{}{}.{}", folder, prefix, lib_name, extension)
        };
        let lib_path = Path::new(&lib_path_string).canonicalize().unwrap();
        let (tx, rx) = channel();
        let (library, loaded_path) = copy_and_load_library(&lib_path_string);
        let mut watcher = raw_watcher(tx).unwrap();
        // Watch the folder and then filter events based on the library path
        watcher.watch(&folder, RecursiveMode::NonRecursive).unwrap();

        HotReloadLib {
            original_lib_path_string: lib_path_string,
            original_lib_path: lib_path,
            loaded_path,
            library: Some(library),
            watch_event_receiver: rx,
            _watcher: watcher,
        }
    }

    pub fn load_symbol<Signature>(&self, symbol_name: &str) -> lib::Symbol<Signature> {
        match self.library {
            Some(ref x) => unsafe {
                x.get(symbol_name.as_bytes())
                    .unwrap_or_else(|_| panic!("Failed to find symbol '{:?}'", symbol_name))
            },
            None => panic!(),
        }
    }

    pub fn update(&mut self) -> bool {
        use notify::op::*;

        let mut should_reload = false;

        for event in self.watch_event_receiver.try_iter() {
            if let RawEvent {
                path: Some(path),
                op: Ok(op),
                cookie: _,
            } = event
            {
                if path.as_path() == self.original_lib_path {
                    // dbg!(op);

                    if op == CREATE {
                        should_reload = true;
                    }
                }
            }
        }

        if should_reload {
            self.library = None; // Work around library not reloading
            fs::remove_file(&self.loaded_path).unwrap();
            let (library, path) = copy_and_load_library(&self.original_lib_path_string);
            self.library = Some(library);
            self.loaded_path = path;
        }

        should_reload
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
        unsafe {
            lib::Library::new(unique_lib_path.as_os_str())
                .unwrap_or_else(|_| panic!("Failed to load library '{:?}'", unique_lib_path))
        },
        unique_lib_path,
    )
}
