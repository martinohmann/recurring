use alloc::boxed::Box;
use core::{error, fmt, ops::Range};
use jiff::{Error as JiffError, civil::DateTime};

/// A macro for conveniently creating adhoc error values.
///
/// This accepts the same arguments as the `format!` macro.
macro_rules! err {
    ($($tt:tt)*) => {{
        crate::error::Error::adhoc(format_args!($($tt)*))
    }}
}

pub(crate) use err;

/// Error type returned by all fallible operations within this crate.
///
/// This type is opaque for the time being and currently does not provide any introspection
/// capabilities apart from its [`Debug`][fmt::Debug] and [`Display`][fmt::Display]
/// implementations.
#[derive(Clone)]
pub struct Error {
    kind: Box<ErrorKind>,
}

impl Error {
    /// Creates a new "ad hoc" error value.
    ///
    /// An ad hoc error value is just an opaque string.
    #[inline(never)]
    #[cold]
    pub(crate) fn adhoc(message: impl fmt::Display) -> Error {
        Error::from(ErrorKind::Adhoc(AdhocError::from_display(message)))
    }

    /// Creates a new error indicating that a `given` value is out of the specified `min..=max`
    /// range.
    #[inline(never)]
    #[cold]
    pub(crate) fn range(given: impl Into<i64>, min: impl Into<i64>, max: impl Into<i64>) -> Error {
        Error::from(ErrorKind::Range(RangeError::new(given, min, max)))
    }

    /// Creates a new error indicating that the end of a datetime range is not strictly greater
    /// than its start.
    #[inline(never)]
    #[cold]
    pub(crate) fn datetime_range(what: &'static str, range: Range<DateTime>) -> Error {
        Error::from(ErrorKind::DateTimeRange(DateTimeRangeError::new(
            what, range,
        )))
    }

    /// Creates a new error from a `jiff` error.
    #[inline(never)]
    #[cold]
    pub(crate) fn jiff(err: JiffError) -> Error {
        Error::from(ErrorKind::Jiff(err))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.kind, f)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Error").field("kind", &self.kind).finish()
    }
}

impl error::Error for Error {}

impl From<JiffError> for Error {
    fn from(err: JiffError) -> Self {
        Error::jiff(err)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error {
            kind: Box::new(kind),
        }
    }
}

/// The underlying kind of [`Error`].
#[derive(Clone, Debug)]
enum ErrorKind {
    /// An ad hoc error that is constructed from anything that implements the `core::fmt::Display`
    /// trait.
    Adhoc(AdhocError),
    /// An error that occurs when a number is not within its allowed range.
    Range(RangeError),
    /// An error indicating that the end of a datetime range is not strictly greater than its
    /// start.
    DateTimeRange(DateTimeRangeError),
    /// An error produced by fallible operations on `jiff` types.
    Jiff(JiffError),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorKind::Adhoc(ref adhoc) => fmt::Display::fmt(adhoc, f),
            ErrorKind::Range(ref range) => fmt::Display::fmt(range, f),
            ErrorKind::DateTimeRange(ref range) => fmt::Display::fmt(range, f),
            ErrorKind::Jiff(ref jiff) => fmt::Display::fmt(jiff, f),
        }
    }
}

/// A generic error message.
#[derive(Clone)]
struct AdhocError {
    message: Box<str>,
}

impl AdhocError {
    /// Creates a new "ad hoc" error value.
    ///
    /// An ad hoc error value is just an opaque string.
    fn from_display(message: impl fmt::Display) -> AdhocError {
        use alloc::string::ToString;
        AdhocError {
            message: message.to_string().into_boxed_str(),
        }
    }
}

impl error::Error for AdhocError {}

impl fmt::Display for AdhocError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.message, f)
    }
}

impl fmt::Debug for AdhocError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.message, f)
    }
}

/// An error that occurs when an input value is out of bounds.
#[derive(Debug, Clone)]
struct RangeError {
    given: i64,
    min: i64,
    max: i64,
}

impl RangeError {
    /// Creates a new error indicating that a `given` value is out of the specified `min..=max`
    /// range.
    fn new(given: impl Into<i64>, min: impl Into<i64>, max: impl Into<i64>) -> RangeError {
        RangeError {
            given: given.into(),
            min: min.into(),
            max: max.into(),
        }
    }
}

impl error::Error for RangeError {}

impl fmt::Display for RangeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let RangeError { given, min, max } = *self;
        write!(
            f,
            "parameter with value {given} \
                 is not in the required range of {min}..={max}",
        )
    }
}

/// An error that occurs when the end of a datetime range is not strictly greater than its start.
#[derive(Debug, Clone)]
struct DateTimeRangeError {
    what: &'static str,
    range: Range<DateTime>,
}

impl DateTimeRangeError {
    /// Creates a new error indicating that the end of a datetime range is not strictly greater
    /// than its start.
    fn new(what: &'static str, range: Range<DateTime>) -> DateTimeRangeError {
        DateTimeRangeError { what, range }
    }
}

impl error::Error for DateTimeRangeError {}

impl fmt::Display for DateTimeRangeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{what} end must be greater than start but got {what} range {start}..{end}",
            what = self.what,
            start = self.range.start,
            end = self.range.end
        )
    }
}
