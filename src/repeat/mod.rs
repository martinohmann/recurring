//! Types for specifying repeat periods.

mod combined;
mod daily;
mod period;
mod timespec;
mod utils;

pub use combined::Combined;
pub use daily::Daily;
use jiff::{Span, ToSpan};
pub use period::Period;
pub use timespec::TimeSpec;

/// Creates a timespec for repeating events.
///
/// Unless further methods are called on the returned `TimeSpec`, this will produce events at every
/// second.
///
/// # Example
///
/// ```
/// use jiff::ToSpan;
/// use recurring::repeat::spec;
///
/// let spec = spec().second(10);
/// ```
#[inline]
pub fn spec() -> TimeSpec {
    TimeSpec::new()
}

/// Creates a period for repeating events.
///
/// # Panics
///
/// Panics if `span` is negative or zero.
///
/// # Example
///
/// ```
/// use jiff::ToSpan;
/// use recurring::repeat::period;
///
/// let every_day_and_a_half = period(1.day().hours(12));
/// ```
#[inline]
pub fn period(span: Span) -> Period {
    Period::new(span)
}

/// Creates an period for repeating events on a per-second basis.
///
/// # Panics
///
/// Panics if `period` is negative or zero.
///
/// # Example
///
/// ```
/// use recurring::repeat::secondly;
///
/// let every_ten_seconds = secondly(10);
/// ```
#[inline]
pub fn secondly<I: ToSpan>(period: I) -> Period {
    Period::new(period.seconds())
}

/// Creates an period for repeating events on a per-minute basis.
///
/// # Panics
///
/// Panics if `period` is negative or zero.
///
/// # Example
///
/// ```
/// use recurring::repeat::minutely;
///
/// let every_thirty_minutes = minutely(30);
/// ```
#[inline]
pub fn minutely<I: ToSpan>(period: I) -> Period {
    Period::new(period.minutes())
}

/// Creates an period for repeating events on a hourly basis.
///
/// # Panics
///
/// Panics if `period` is negative or zero.
///
/// # Example
///
/// ```
/// use recurring::repeat::hourly;
///
/// let every_twelve_hours = hourly(12);
/// ```
#[inline]
pub fn hourly<I: ToSpan>(period: I) -> Period {
    Period::new(period.hours())
}

/// Creates an period for repeating events on a daily basis.
///
/// # Panics
///
/// Panics if `period` is negative or zero.
///
/// # Example
///
/// ```
/// use recurring::repeat::daily;
///
/// let every_two_days = daily(2);
/// ```
#[inline]
pub fn daily<I: ToSpan>(period: I) -> Daily {
    Daily::new(period)
}

/// Creates an period for repeating events on a monthly basis.
///
/// # Panics
///
/// Panics if `period` is negative or zero.
///
/// # Example
///
/// ```
/// use recurring::repeat::monthly;
///
/// let every_three_months = monthly(3);
/// ```
#[inline]
pub fn monthly<I: ToSpan>(period: I) -> Period {
    Period::new(period.months())
}

/// Creates an period for repeating events on a yearly basis.
///
/// # Panics
///
/// Panics if `period` is negative or zero.
///
/// # Example
///
/// ```
/// use recurring::repeat::yearly;
///
/// let every_five_years = yearly(5);
/// ```
#[inline]
pub fn yearly<I: ToSpan>(period: I) -> Period {
    Period::new(period.years())
}
