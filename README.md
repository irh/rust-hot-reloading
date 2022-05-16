# An Example of Hot-Reloading in Rust

## Hot-Reloading?

Hot-reloading is a technique that allows for rapid iteration during the development of an application.

The core idea is that by encapsulating core logic in dynamic libraries, and then carefully managing application state, we can reload a newly compiled library while the application continues to run and have changes take immediate effect. This allows for the type of rapid iteration of changes in a compiled language that might typically be associated with scripting languages.

I learned about this technique recently while watching the excellent [Handmade Hero](https://handmadehero.org) series (see episodes 21-23), and figured I'd give it a try in my own projects. There are already [some](https://github.com/draivin/rust-hotswap) [examples](https://github.com/porglezomp-misc/live-reloading-rs) out there showing how to achieve this in Rust, but they seem to do either more than I need, or I didn't have any luck in making them work in my project.

Plus, I wanted to explore how to extend the idea to build a deployable, statically linked application once past the initial development phase, so I've ended up with the approach that you can find presented here. I've tried to boil it down into a minimal template that should hopefully be useful as a starting point for new projects.

## Trying out the example application

- The test scripts use [Cargo Watch](https://github.com/passcod/cargo-watch), which you can install by running `cargo install cargo-watch`.
- In separate terminals, run both the `watch-app.sh` and the `watch-libs.sh` scripts.
  - `watch-app.sh` takes care of updating the main application when necessary, and `watch-libs.sh` looks after updating the dynamic libraries.
- In the application output, you will see a series of numbers being printed.
- Edit the `view_state` function in `view/src/lib.rs` to modify the way that the numbers are printed.
- After saving the file, the `view` library will be automatically recompiled and loaded into the application. You should see the application output change immediately.
- Now try editing the `update_state` function in `update/src/lib.rs` to modify the way that the numbers are manipulated.
- Editing the main application in the `app` folder, or the `shared` library will cause the `watch-app.sh` script to relaunch the application.

## Implementation

This example has two hot-reloaded libraries, `update` and `view`, with a single `shared` library which is shared between the hot-reloaded libraries and the main application. This structure is only intended as a starting point, and the technique should be usable with almost any application structure.

The `hot_reload_lib` crate contains the `HotReloadLib` object which takes care of reloading a library whenever it changes. No effort is made to expose functions elegantly, you simply request a symbol with a specific signature from the library, and expect it to be present. This is a bit quick and dirty, but it's good enough for my purposes.

By default, hot-reloading of libraries is disabled and the libraries get statically linked, and the `hot_reload_libs` feature flag enables hot-reloading when needed. This allows the use of hot-reloading during development, and then a regular `cargo build --release` will produce a statically linked deployable application.

### Platform Support

This approach works well on Linux, and _used_ to work well on macOS, but since I first worked on this 'something' has changed and now libraries will be reloaded several times after each file write. I don't currently make use of this project and I don't have spare time to investigate, but at least the general goal of having hot-reloaded libraries still works, and reloading libraries with this approach is intended to be lightweight, so the additional reloads shouldn't be problematic.

I expect that the approach taken in this project should apply to other OSs, please let me know if you have success with it on another platform, and feel free to open a PR.

### State Management

In this approach to hot-reloading, libraries must contain no internal state. All state needed by the hot-reloaded libraries is declared externally, owned by the application and then passed into each library function call.

In this example, a `shared` library (which as its name suggests, is shared between the application and the libraries) declares a `State` struct which is instantiated by the application on launch. Any changes to the `shared` library cause the application to be relaunched by the `watch-app.sh` script.

### Avoiding Thread-Local Storage

Depending on your operating system, loaded dynamic libraries might be cached, which prevents libraries being reloaded while an application is running. To get around the caching, libraries get copied to a unique location before they're loaded.

Note that in this implementation, hot-reloading doesn't work at all if the library uses Thread Local Storage (TLS). This caused me headaches for quite some time figuring out why reloading would sometimes stop working. Any use of TLS will prevent hot-reloading from working, even with the unique copy trick. ~~I'm not clear on why this should be the case, if you happen to know what's going on here please let me know!~~ [@fasterthanlime](https://github.com/fasterthanlime) has explained this issue in depth in [this blog post](https://fasterthanli.me/articles/so-you-want-to-live-reload-rust), which is well worth a read!

Without an implementation of the solution proposed in the blog post linked above, to work around this issue any use of TLS need to be in the main application, and functionality that depends on it should be hidden behind an interface. An example of how to do this is shown with the `Rng` trait in the `shared` library. The `thread_rng` feature of the `rand` package uses TLS and prevents hot-reloading from working, so in this example the `rand` crate is only linked into the main application.

This encapsulation of dependencies has the additional benefit of minimizing the compile times of your hot-reloaded libraries. The burden of compiling the dependencies rests on the main application, so recompiles and reloads of the libraries can take place as quickly as possible.
