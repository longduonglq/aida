use std::borrow::Borrow;
use std::fmt::{Debug, Formatter};
use std::ops::{Add, Sub};
use fraction::Integer;

pub struct PInterval<T>
{
    start: T,
    end: T,
    length: T
}

impl<T> PInterval<T>
where T:
    Sub<Output = T> + Add<Output = T> +
    PartialOrd<i32> + Ord +
    Copy + Debug
{
    fn from_end_points(_start: T, _end: T) -> Self {
        Self {
            start: _start,
            end: _end,
            length: _end - _start
        }
    }

    fn from_start_and_length(_start: T, _length: T) -> Self {
        Self {
            start: _start,
            end: _start + _length,
            length: _length
        }
    }

    fn set_start_keep_length(&mut self, _start: T)  {
        self.start = _start;
        self.end = self.start + self.length;
    }

    fn set_start_keep_end(&mut self, _start: T) {
        self.start = _start;
        self.length = self.end - self.start;
        assert!(self.length >= 0);
    }

    fn set_length_keep_start(&mut self, _length: T) {
        self.length = _length;
        self.end = self.start + self.length;
    }

    fn set_length_keep_end(&mut self, _length: T) {
        self.length = _length;
        self.start = self.end - self.length;
    }

    fn set_end_keep_start(&mut self, _end: T) {
        self.end = _end;
        self.length = self.end - self.start;
    }

    fn set_end_keep_length(&mut self, _end: T) {
        self.end = _end;
        self.start = self.end - self.length;
    }

    fn does_overlap_with(&self, other: &Self) -> bool {
        self.start < other.end && other.start < self.end
    }

    fn is_disjoint_with(&self, other: &Self) -> bool {
        !self.does_overlap_with(other)
    }

    fn does_swallow(&self, other: &Self) -> bool {
        self.start <= other.start && self.end >= other.end
    }

    fn is_swallowed_by(&self, other: &Self) -> bool {
        other.start <= self.start && other.end >= self.end
    }

    fn get_intersection_with(&self, other: &Self) -> Self {
        Self::from_end_points(
            core::cmp::max(self.start, other.start),
            core::cmp::min(self.end, other.end)
        )
    }
}

impl<T: PartialEq> PartialEq for PInterval<T> {
    fn eq(&self, rhs: &Self) -> bool {
        (self.start == rhs.start) &&
        (self.end == rhs.end) &&
        (self.length == rhs.length)
    }
}

impl<T: Debug> Debug for PInterval<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("PInterval[{:?}, {:?})", self.start, self.end));
        Ok(())
    }
}
