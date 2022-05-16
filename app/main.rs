extern crate rand;
extern crate shared;

#[cfg(feature = "hot_reload_libs")]
extern crate hot_reload_lib;

#[cfg(not(feature = "hot_reload_libs"))]
extern crate update;

#[cfg(not(feature = "hot_reload_libs"))]
extern crate view;

#[cfg(feature = "hot_reload_libs")]
use hot_reload_lib::HotReloadLib;

use rand::Rng;
use std::{thread, time};

struct RngImpl;

impl shared::Rng for RngImpl {
    fn gen_range(&mut self, low: i32, high: i32) -> i32 {
        rand::thread_rng().gen_range(low, high)
    }
}

#[cfg(feature = "hot_reload_libs")]
struct HotReloadLibs {
    update: HotReloadLib,
    view: HotReloadLib,
}

#[cfg(feature = "hot_reload_libs")]
impl HotReloadLibs {
    fn new(hot_reload_libs_folder: &str) -> Self {
        Self {
            update: HotReloadLib::new(hot_reload_libs_folder, "update"),
            view: HotReloadLib::new(hot_reload_libs_folder, "view"),
        }
    }

    fn update_libs(&mut self) {
        self.update.update();
        self.view.update();
    }
}

struct Application {
    state: shared::State,

    #[cfg(feature = "hot_reload_libs")]
    libs: HotReloadLibs,
}

impl Application {
    fn new(_hot_reload_libs_folder: &str) -> Application {
        let rng = Box::new(RngImpl{});

        Application {
            state: shared::State::new(rng),

            #[cfg(feature = "hot_reload_libs")]
            libs: HotReloadLibs::new(_hot_reload_libs_folder),
        }
    }

    #[cfg(feature = "hot_reload_libs")]
    fn update_state(&mut self) {
        self.libs
            .update
            .load_symbol::<fn(&mut shared::State)>("update_state")(&mut self.state);
    }

    #[cfg(not(feature = "hot_reload_libs"))]
    fn update_state(&mut self) {
        update::update_state(&mut self.state);
    }

    #[cfg(feature = "hot_reload_libs")]
    fn view_state(&self) {
        self.libs
            .view
            .load_symbol::<fn(&shared::State)>("view_state")(&self.state);
    }

    #[cfg(not(feature = "hot_reload_libs"))]
    fn view_state(&self) {
        view::view_state(&self.state);
    }
}

fn main() {

    let mut app = Application::new("target/debug");

    println!("Starting loop");
    loop {
        #[cfg(feature = "hot_reload_libs")]
        app.libs.update_libs();

        app.update_state();
        app.view_state();

        thread::sleep(time::Duration::from_millis(1000));
    }
}
