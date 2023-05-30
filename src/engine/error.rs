use std::fmt;

#[derive(Debug, Clone)]
pub struct NotConvergedError;

impl fmt::Display for NotConvergedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Couldn't converge simulation.")
    }
}
