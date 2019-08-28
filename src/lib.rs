pub mod agaritilesets;
pub mod context;
pub mod form;
pub mod judge;
pub mod tile;
pub mod tiles;
pub mod tileset;
pub mod tilesets;
mod utils;

#[cfg(test)]
pub mod logger {
    use env_logger::{Builder, Target};
    use std::sync::Once;

    static INIT_LOGGER: Once = Once::new();

    pub fn init_once() {
        INIT_LOGGER.call_once(|| {
            let mut builder = Builder::from_default_env();
            builder.target(Target::Stdout);
            builder.init();
        });
    }
}
