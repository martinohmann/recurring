use jiff::{ToSpan, civil::date};
use pretty_assertions::assert_eq;
use recurring::{
    Event, Series,
    pattern::{Cron, Interval, daily},
};

macro_rules! assert_err {
    ($expr:expr, $msg:literal $(,)?) => {
        assert_eq!(($expr).unwrap_err().to_string(), $msg);
    };
}

#[test]
fn event_errors() {
    let start = date(2025, 1, 1).at(0, 0, 0, 0);

    assert_err!(
        Event::try_new(start, start),
        "event end must be greater than start but got event range 2025-01-01T00:00:00..2025-01-01T00:00:00",
    );
}

#[test]
fn series_errors() {
    let start = date(2025, 1, 1).at(0, 0, 0, 0);

    assert_err!(
        Series::try_new(start..start, daily(1)),
        "series end must be greater than start but got series range 2025-01-01T00:00:00..2025-01-01T00:00:00",
    );

    assert_err!(
        Series::new(start.., daily(1))
            .with()
            .event_duration(-1.second())
            .build(),
        "event duration must be positive or zero but got -PT1S",
    );

    assert_err!(
        Series::new(start.., daily(1)).with().end(start).build(),
        "series end must be greater than start but got series range 2025-01-01T00:00:00..2025-01-01T00:00:00",
    );
}

#[test]
fn interval_errors() {
    assert_err!(
        Interval::try_new(0.seconds()),
        "interval must be positive, non-zero and must not include sub-second units but got PT0S",
    );

    assert_err!(
        Interval::try_new(1.second().nanoseconds(10)),
        "interval must be positive, non-zero and must not include sub-second units but got PT1.00000001S",
    );
}

#[test]
fn cron_errors() {
    assert_err!(
        Cron::new().try_second(-10),
        "parameter with value -10 is not in the required range of 0..=59",
    );

    assert_err!(
        Cron::new().try_years([2025, 10000]),
        "parameter with value 10000 is not in the required range of -9999..=9999",
    );
}
