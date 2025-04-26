#[derive(Debug, Clone)]
pub enum Error {
    InvalidSeriesEnd,
    InvalidEventEnd,
    InvalidEventDuration,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidEventEnd => f.write_str("event end must be greater than start"),
            Error::InvalidSeriesEnd => f.write_str("series end must be greater than start"),
            Error::InvalidEventDuration => {
                f.write_str("event duration must be non-zero and positive")
            }
        }
    }
}

impl std::error::Error for Error {}
