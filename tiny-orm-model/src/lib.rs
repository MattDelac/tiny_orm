mod errors;
pub use crate::errors::TinyOrmError;

#[cfg(feature = "set-option")]
mod set_option;
#[cfg(feature = "set-option")]
pub use crate::set_option::SetOption;
