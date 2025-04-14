use std::{
    cmp::{self, Ordering},
    usize,
};

#[derive(Debug, Clone)]
pub enum EitherOrBoth<T> {
    Left(T),
    Right(T),
    Both(T),
}

enum State<T> {
    LeftItem(T),
    RightItem(T),
}

/// Given a pair of iterators in sorted order, iterate over the items, such that
/// we discover which of those items are present on the left, on the right,
/// or both. For instance, given [1, 2, 4] and [1, 3, 4], we'll get (Both(1),
/// Left(2), Right(3), Both(4)).
///
/// The overall ordering will remain sorted.
pub struct SortedPairsIter<I: Iterator> {
    left: I,
    right: I,
    state: Option<State<I::Item>>,
}

impl<I: Iterator> SortedPairsIter<I>
where
    I::Item: Ord,
{
    pub fn new(left: I, right: I) -> Self {
        Self {
            left,
            right,
            state: None,
        }
    }
}

impl<I> Iterator for SortedPairsIter<I>
where
    I: Iterator,
    I::Item: Ord,
{
    type Item = EitherOrBoth<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        match match self.state.take() {
            None => (self.left.next(), self.right.next()),
            Some(State::LeftItem(item)) => (Some(item), self.right.next()),
            Some(State::RightItem(item)) => (self.left.next(), Some(item)),
        } {
            (None, None) => None,
            (Some(left), None) => Some(EitherOrBoth::Left(left)),
            (None, Some(right)) => Some(EitherOrBoth::Right(right)),
            (Some(left), Some(right)) => {
                let (state, item) = match Ord::cmp(&left, &right) {
                    Ordering::Equal => return Some(EitherOrBoth::Both(left)),
                    Ordering::Less => (State::RightItem(right), EitherOrBoth::Left(left)),
                    Ordering::Greater => (State::LeftItem(left), EitherOrBoth::Right(right)),
                };
                self.state = Some(state);
                Some(item)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (left_min, left_max) = self.left.size_hint();
        let (right_min, right_max) = self.right.size_hint();

        // We could do a more complex computation here (related to more
        // correctly accounting for the save item in `self.state`), but I just
        // don't feel like it right now

        // Min: in the best case, there's no saved item, and all the remaining
        // elements in both iterators are identical (or one is a subset of the
        // other)
        let min = cmp::max(left_min, right_min);

        // Max: in the worse case, there's a total disjointness between left
        // and right, so we add them together, and add 1 for the element that
        // might have been saved.
        let max = left_max.and_then(|left_max| left_max.checked_add(right_max?)?.checked_add(1));

        (min, max)
    }
}
