use std::{
    fmt::{Debug, Display},
    io,
};

pub type Result<T> = std::result::Result<T, Error>;

struct Error {
    err: ErrorKind,
}

enum ErrorKind {
    Serde(serde_json::Error),
    PathIO(io::Error),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.err, f)
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::Serde(serde) => write!(f, "IO Serde Error: {}", serde),
            ErrorKind::PathIO(io) => write!(f, "Path IO Error: {}", io),
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Crypto Alertabot Error: {}", self.err.to_string())
    }
}

impl From<ErrorKind> for Error {
    fn from(err: ErrorKind) -> Self {
        Self { err }
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(serde: serde_json::error::Error) -> Self {
        ErrorKind::Serde(serde).into()
    }
}

impl From<io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        ErrorKind::PathIO(err).into()
    }
}
