/// Error type returned by all fallible operations within this crate.
///
/// This type is opaque for the time being and currently does not provide any introspection
/// capabilities apart from its [`Debug`][core::fmt::Debug] and [`Display`][core::fmt::Display]
/// implementations.
#[derive(Clone)]
pub struct Error {
    inner: ErrorKind,
}

impl Error {
    pub(crate) fn from_kind(kind: ErrorKind) -> Error {
        Error { inner: kind }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.inner {
            ErrorKind::InvalidBounds => f.write_str("end must be greater than start"),
            ErrorKind::InvalidEventDuration => {
                f.write_str("event duration must be positive or zero")
            }
            ErrorKind::InvalidPeriod => {
                f.write_str("period must be positive, non-zero and not include sub-second units")
            }
            ErrorKind::Jiff(err) => err.fmt(f),
        }
    }
}

impl core::fmt::Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Error").field(&self.inner).finish()
    }
}

impl core::error::Error for Error {}

impl From<jiff::Error> for Error {
    fn from(err: jiff::Error) -> Self {
        Error::from_kind(ErrorKind::Jiff(err))
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error::from_kind(kind)
    }
}

#[derive(Clone, Debug)]
pub(crate) enum ErrorKind {
    InvalidEventDuration,
    InvalidPeriod,
    InvalidBounds,
    Jiff(jiff::Error),
}
