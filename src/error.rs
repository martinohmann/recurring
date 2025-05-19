/// Error type returned by all fallible operations within this crate.
///
/// This type is opaque for the time being and currently does not provide any introspection
/// capabilities apart from its [`Debug`][core::fmt::Debug] and [`Display`][core::fmt::Display]
/// implementations.
#[derive(Clone)]
pub struct Error {
    kind: ErrorKind,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.kind {
            ErrorKind::InvalidBounds => f.write_str("end must be greater than start"),
            ErrorKind::InvalidEventDuration => {
                f.write_str("event duration must be positive or zero")
            }
            ErrorKind::InvalidInterval => {
                f.write_str("interval must be positive, non-zero and not include sub-second units")
            }
            ErrorKind::Jiff(err) => err.fmt(f),
            ErrorKind::OutOfBounds => f.write_str("value out of bounds"),
        }
    }
}

impl core::fmt::Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Error").field(&self.kind).finish()
    }
}

impl core::error::Error for Error {}

impl From<jiff::Error> for Error {
    fn from(err: jiff::Error) -> Self {
        Error::from(ErrorKind::Jiff(err))
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error { kind }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum ErrorKind {
    InvalidEventDuration,
    InvalidInterval,
    InvalidBounds,
    Jiff(jiff::Error),
    OutOfBounds,
}
