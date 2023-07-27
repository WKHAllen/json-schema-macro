#![forbid(unsafe_code)]

#[doc(hidden)]
pub mod __internal {
    pub use serde_json;
}

pub use json_schema_macro_macros::*;
