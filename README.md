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

## Example: `Series` with cron pattern

```rust
use recurring::{Event, Series};
use recurring::pattern::cron;
use jiff::civil::date;

let start = date(2025, 7, 1).at(12, 0, 0, 0);
let pattern = cron().hours([12, 16]).minute(5).seconds([10, 20]);
let series = Series::new(start.., pattern);

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

## Example: `Series` with interval pattern from a `jiff` type

```rust
use recurring::{Event, Series, ToSeries};
use recurring::pattern::daily;
use jiff::civil::date;

let start = date(2025, 7, 1).at(12, 0, 0, 0);
let series = start.to_series(daily(2)).unwrap();

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

## Example: `Series` with event duration and multiple recurrence patterns

```rust
use recurring::{Combine, Event, Series};
use recurring::pattern::cron;
use jiff::{civil::date, ToSpan};

let start = date(2025, 6, 30).at(0, 0, 0, 0);
let daily_around_lunch = cron().hour(12).minute(5).second(10);
let first_of_month_in_the_morning = cron().day(1).hour(8).minute(45).second(0);
let pattern = daily_around_lunch.and(first_of_month_in_the_morning);

let series = Series::builder(start.., pattern)
    .event_duration(1.hour())
    .build()
    .unwrap();

let events: Vec<Event> = series.iter().take(4).collect();

assert_eq!(
    events,
    [
        Event::new(
            date(2025, 6, 30).at(12, 5, 10, 0),
            date(2025, 6, 30).at(13, 5, 10, 0),
        ),
        Event::new(
            date(2025, 7, 1).at(8, 45, 0, 0),
            date(2025, 7, 1).at(9, 45, 0, 0),
        ),
        Event::new(
            date(2025, 7, 1).at(12, 5, 10, 0),
            date(2025, 7, 1).at(13, 5, 10, 0),
        ),
        Event::new(
            date(2025, 7, 2).at(12, 5, 10, 0),
            date(2025, 7, 2).at(13, 5, 10, 0),
        ),
    ]
);
```

## Example: Querying a `Series` for events before, after or close to a datetime

```rust
use recurring::{Combine, Event, Series};
use recurring::pattern::cron;
use jiff::{civil::date, ToSpan};

let start = date(2025, 7, 1).at(0, 0, 0, 0);
let end = date(2025, 7, 10).at(0, 0, 0, 0);
let pattern = cron().hours([12, 18]).minutes([5, 10]).second(0);

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

## License

The source code of recurring is licensed under either of [Apache License,
Version 2.0](LICENSE-APACHE.md) or [MIT license](LICENSE-MIT) at your option.
