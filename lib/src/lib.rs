//! Types and functions for code that defines and/or uses `#[typeshare]`
//! types.

pub use typeshare_annotation::typeshare;

mod integer;
pub use integer::{usize_from_u53_saturated, I54, U53};
