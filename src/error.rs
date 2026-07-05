use std::fmt;

#[derive(Debug)]
pub enum Error {
    ConfigIo(String),
    ConfigParse(String),
    NotARepo(String),
    DuplicatePath(String),
    NotFound(String),
    Git(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ConfigIo(msg) => write!(f, "config I/O error: {msg}"),
            Error::ConfigParse(msg) => write!(f, "config parse error: {msg}"),
            Error::NotARepo(path) => write!(f, "not a git repository: {path}"),
            Error::DuplicatePath(path) => write!(f, "path already registered: {path}"),
            Error::NotFound(name) => write!(f, "no project found matching: {name}"),
            Error::Git(msg) => write!(f, "git error: {msg}"),
        }
    }
}

impl std::error::Error for Error {}
