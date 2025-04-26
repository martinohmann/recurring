#[derive(Debug, Clone)]
pub enum Error {
    InvalidSeriesEnd,
    InvalidEventEnd,
    InvalidEventDuration,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::InvalidEventEnd => f.write_str("event end must be greater than start"),
            Error::InvalidSeriesEnd => f.write_str("series end must be greater than start"),
            Error::InvalidEventDuration => f.write_str("event duration must be positive"),
        }
    }
}

impl core::error::Error for Error {}
