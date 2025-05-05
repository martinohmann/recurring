#[derive(Debug, Clone)]
pub enum Error {
    InvalidEventDuration,
    InvalidInterval,
    InvalidBounds,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::InvalidBounds => f.write_str("end must be greater than start"),
            Error::InvalidEventDuration => f.write_str("event duration must be positive or zero"),
            Error::InvalidInterval => f.write_str("interval must be positive and non-zero"),
        }
    }
}

impl core::error::Error for Error {}
