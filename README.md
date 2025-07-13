# recurring

[![Build Status](https://github.com/martinohmann/recurring/workflows/ci/badge.svg)](https://github.com/martinohmann/recurring/actions?query=workflow%3Aci)
[![crates.io](https://img.shields.io/crates/v/recurring)](https://crates.io/crates/recurring)
[![docs.rs](https://img.shields.io/docsrs/recurring)](https://docs.rs/recurring)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

An event series implementation based on
[`jiff`](https://docs.rs/jiff/latest/jiff/) which provides a flexible
alternative to `jiff`'s builtin series iterators provided by the
[`DateTime::series`](https://docs.rs/jiff/latest/jiff/civil/struct.DateTime.html#method.series),
[`Date::series`](https://docs.rs/jiff/latest/jiff/civil/struct.DateTime.html#method.series)
and
[`Time::series`](https://docs.rs/jiff/latest/jiff/civil/struct.Time.html#method.series)
methods.

The essential types in this crate are:

- [`Pattern`](https://docs.rs/recurring/latest/recurring/trait.Pattern.html): a
  trait implemented by recurrence patterns to yield a `DateTime` before, after
  or close to a given instant.
- [`Event`](https://docs.rs/recurring/latest/recurring/struct.Event.html): an
  event with a start and optional end date.
- [`Series`](https://docs.rs/recurring/latest/recurring/struct.Series.html):
  produces `Event`s following a recurrence `Pattern`.

## Features

- Interval-based and cron-based [recurrence patterns](https://docs.rs/recurring/latest/recurring/pattern/index.html)
- Support for optional event duration to produce [series
  events](https://docs.rs/recurring/latest/recurring/struct.Event.html) with
  start and end date.
- Querying the
  [`Series`](https://docs.rs/recurring/latest/recurring/struct.Series.html) for
  events.
  [before](https://docs.rs/recurring/latest/recurring/struct.Series.html#method.get_previous_before),
  [after](https://docs.rs/recurring/latest/recurring/struct.Series.html#method.get_next_after)
  or
  [close](https://docs.rs/recurring/latest/recurring/struct.Series.html#method.get_closest_to)
  to a user-defined datetime.
- Finding series events
  [containing](https://docs.rs/recurring/latest/recurring/struct.Series.html#method.get_containing)
  a given datetime.
- [Composition](https://docs.rs/recurring/latest/recurring/trait.Combine.html)
  of events following different recurrence patterns into a single series.
- [Splitting](https://docs.rs/recurring/latest/recurring/struct.Series.html#method.split_off) series at a cutoff point.
- Iterating over a [sub-range](https://docs.rs/recurring/latest/recurring/struct.Series.html#method.range) of a series.

## Examples

### Series with interval-based recurrence pattern

Construct a `Series` yielding events on a fixed interval.

```rust
use recurring::{Event, Series};
use recurring::pattern::daily;
use jiff::civil::date;

// Every two days.
let pattern = daily(2);

// The pattern is relative to the series start.
let start = date(2025, 7, 1).at(12, 0, 0, 0);

let series = Series::new(start.., pattern);

let events: Vec<Event> = series.iter().take(5).collect();

assert_eq!(
    events,
    [
        Event::at(date(2025, 7, 1).at(12, 0, 0, 0)),
        Event::at(date(2025, 7, 3).at(12, 0, 0, 0)),
        Event::at(date(2025, 7, 5).at(12, 0, 0, 0)),
        Event::at(date(2025, 7, 7).at(12, 0, 0, 0)),
        Event::at(date(2025, 7, 9).at(12, 0, 0, 0)),
    ]
);
```

### Series with cron-based recurrence pattern

Construct a `Series` yielding events according to a cron schedule.

This also showcases the `ToSeries` trait which provides the `.to_series()`
method for builtin `jiff` types like `Zoned`, `DateTime` and `Date`.

```rust
use recurring::{Event, Series, ToSeries};
use recurring::pattern::cron;
use jiff::civil::date;

// Create a cron pattern which yields events every day at 12:05:10, 12:05:20,
// 16:05:10 and 16:05:20.
let pattern = cron().hours([12, 16]).minute(5).seconds([10, 20]);

let start = date(2025, 7, 1).at(12, 0, 0, 0);

// `.to_series()` is provided by the `ToSeries` trait.
let series = start.to_series(pattern).unwrap();

let events: Vec<Event> = series.iter().take(5).collect();

assert_eq!(
    events,
    [
        Event::at(date(2025, 7, 1).at(12, 5, 10, 0)),
        Event::at(date(2025, 7, 1).at(12, 5, 20, 0)),
        Event::at(date(2025, 7, 1).at(16, 5, 10, 0)),
        Event::at(date(2025, 7, 1).at(16, 5, 20, 0)),
        Event::at(date(2025, 7, 2).at(12, 5, 10, 0)),
    ]
);
```

### Querying a series for events

The `Series` type has various methods to query for events. It provides methods
to find the event before, after or closest to a given instant and more. The
example below showcases some of these methods.

```rust
use recurring::{Combine, Event, Series};
use recurring::pattern::cron;
use jiff::{civil::date, ToSpan};

// Create a cron pattern which yields events every day at 12:05:00, 12:10:00,
// 18:05:00 and 18:10:00.
let pattern = cron().hours([12, 18]).minutes([5, 10]).second(0);

// In addition to the series start date, we also specify an end in this
// example. Series bounds can be open or closed on both sides.
let start = date(2025, 7, 1).at(0, 0, 0, 0);
let end = date(2025, 7, 10).at(0, 0, 0, 0);

let series = Series::new(start..end, pattern);

assert_eq!(
    series.get_next_after(start),
    Some(Event::at(date(2025, 7, 1).at(12, 5, 0, 0))),
);

assert_eq!(
    series.get_previous_before(end),
    Some(Event::at(date(2025, 7, 9).at(18, 10, 0, 0))),
);

assert_eq!(
    series.get_closest_to(date(2025, 7, 5).at(0, 0, 0, 0)),
    Some(Event::at(date(2025, 7, 4).at(18, 10, 0, 0))),
);
```

### Advanced usage

The following example shows some more advanced features.

A series can yield events from multiple recurrence patterns, e.g. if you have
events occurring at a fixed interval with occasional exceptions.

Additionally, the example below also shows how a `Series` can also be
configured to return `Event`s that have a start and end date by configuring an
event duration.

Finally, we use the `.range()` method of `Series` to iterate over a sub-range
of the series events.

```rust
use recurring::{Combine, Event, Series};
use recurring::pattern::{cron, daily};
use jiff::{civil::date, ToSpan};

// We have a cron-based pattern...
let daily_around_lunch = cron().hour(12).minute(5).second(10);
// ...and an interval-based pattern
let every_two_days = daily(2);
// ...and construct a pattern which combines the two into a single pattern.
let pattern = daily_around_lunch.and(every_two_days);

// The interval-based `every_two_days` pattern is relative to the series start.
let series_start = date(2025, 6, 28).at(8, 0, 0, 0);

let series = Series::builder(series_start.., pattern)
    .event_duration(1.hour()) // Series events span 1 hour.
    .build()
    .unwrap();

// We iterate over the events starting 2 days after the series start.
let range_start = series_start + 48.hours();
let events: Vec<Event> = series.range(range_start..).take(4).collect();

assert_eq!(
    events,
    [
        Event::new(
            date(2025, 6, 30).at(8, 0, 0, 0),
            date(2025, 6, 30).at(9, 0, 0, 0),
        ),
        Event::new(
            date(2025, 6, 30).at(12, 5, 10, 0),
            date(2025, 6, 30).at(13, 5, 10, 0),
        ),
        Event::new(
            date(2025, 7, 1).at(12, 5, 10, 0),
            date(2025, 7, 1).at(13, 5, 10, 0),
        ),
        Event::new(
            date(2025, 7, 2).at(8, 0, 0, 0),
            date(2025, 7, 2).at(9, 0, 0, 0),
        ),
    ]
);
```

## License

The source code of recurring is licensed under either of [Apache License,
Version 2.0](LICENSE-APACHE.md) or [MIT license](LICENSE-MIT) at your option.
