use std::fmt;

#[derive(Debug)]
pub enum Error {
    Panic,
    Uncalled,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Panic => write!(f, "The fn which was meant to resolve the future panicked"),
            Self::Uncalled => write!(f, "The fn which was meant to resolve the future was not called"),
        }
    }
}

impl std::error::Error for Error { }