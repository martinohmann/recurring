mod common;

use common::{series_take, series_take_rev};
use jiff::civil::{DateTime, date};
use recurring::repeat::{hourly, spec};
use recurring::{Combine, Event};

#[test]
fn combined() {
    let start = date(2025, 1, 1).at(12, 0, 0, 0);
    let end = DateTime::MAX;
    let repeat = spec()
        .hour(10)
        .minute(30)
        .second(0)
        .and(spec().hour(12).minute(45).second(30))
        .and(hourly(6));

    assert_eq!(
        series_take(start..end, repeat.clone(), 10),
        vec![
            Event::at(date(2025, 1, 1).at(12, 0, 0, 0)),
            Event::at(date(2025, 1, 1).at(12, 45, 30, 0)),
            Event::at(date(2025, 1, 1).at(18, 0, 0, 0)),
            Event::at(date(2025, 1, 2).at(0, 0, 0, 0)),
            Event::at(date(2025, 1, 2).at(6, 0, 0, 0)),
            Event::at(date(2025, 1, 2).at(10, 30, 0, 0)),
            Event::at(date(2025, 1, 2).at(12, 0, 0, 0)),
            Event::at(date(2025, 1, 2).at(12, 45, 30, 0)),
            Event::at(date(2025, 1, 2).at(18, 0, 0, 0)),
            Event::at(date(2025, 1, 3).at(0, 0, 0, 0)),
        ]
    );

    assert_eq!(
        series_take_rev(start..end, repeat, 10),
        vec![
            Event::at(date(9999, 12, 31).at(18, 0, 0, 0)),
            Event::at(date(9999, 12, 31).at(12, 45, 30, 0)),
            Event::at(date(9999, 12, 31).at(12, 0, 0, 0)),
            Event::at(date(9999, 12, 31).at(10, 30, 0, 0)),
            Event::at(date(9999, 12, 31).at(6, 0, 0, 0)),
            Event::at(date(9999, 12, 31).at(0, 0, 0, 0)),
            Event::at(date(9999, 12, 30).at(18, 0, 0, 0)),
            Event::at(date(9999, 12, 30).at(12, 45, 30, 0)),
            Event::at(date(9999, 12, 30).at(12, 0, 0, 0)),
            Event::at(date(9999, 12, 30).at(10, 30, 0, 0)),
        ]
    );
}
