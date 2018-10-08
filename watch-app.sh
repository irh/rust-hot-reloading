cargo build --all --features hot_reload_libs \
  && RUST_BACKTRACE=1 cargo watch -i "*/update/**" -i "*/view/**" -x "run --features hot_reload_libs"
