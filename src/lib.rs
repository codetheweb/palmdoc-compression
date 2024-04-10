#[cfg(feature = "calibre")]
pub mod calibre;
pub mod palmdoc;

pub use palmdoc::*;

mod hashtable;
mod window;
