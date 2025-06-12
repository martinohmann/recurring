use crate::series::{Range, Series};
use crate::{Event, Pattern};

/// An iterator over the events of a [`Series`].
///
/// This struct is created by the [`.iter()`][Series::iter] method of a `Series`. See its
/// documentation for more.
#[derive(Debug, Clone)]
pub struct Iter<'a, P> {
    iter: Range<'a, P>,
}

impl<'a, P: Pattern> Iter<'a, P> {
    pub(crate) fn new(series: &'a Series<P>) -> Iter<'a, P> {
        Iter {
            iter: Range::new(&series.core, series.range),
        }
    }
}

impl<P> Iterator for Iter<'_, P>
where
    P: Pattern,
{
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<P> DoubleEndedIterator for Iter<'_, P>
where
    P: Pattern,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}
