#![warn(missing_docs)]
/// constructs a square of crossings. mostly for debugging purposes
mod build_grid;
/// default implementations (TODO: move here)
pub mod simple;
/// the most important traits are seperated
pub mod traits;
/// put build_grid in a submodule
pub mod debug {
    pub use super::build_grid::*;
}
