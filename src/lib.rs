
pub mod bytesbuffer;
pub mod resultenum;
pub mod dns;

use std::io::{Error,ErrorKind};

pub fn default_error() -> Error {
    Error::new(ErrorKind::Other, "End of buffer")
}


pub fn error_with_jumps(max:i32) -> Error {
    Error::new(ErrorKind::Other,format!("Limit of {} jumps exceeded", max))
}

pub fn error_for_custom(msg: &str) -> Error {
    Error::new(ErrorKind::Other,msg)
}