use std::ops::{Range, RangeFrom, RangeTo, RangeFull, RangeToInclusive, RangeInclusive};
use std::fmt::Display;
use std::ptr;

extern crate num_traits;
use self::num_traits::sign::Unsigned;
use self::num_traits::Zero;

// XXX Why does all of this need the Display trait ??
// Similar: std::collections::range::RangeArgument
// See also the `odds` crate.
pub trait IntoIndexRange<T=usize> where T: Clone + PartialOrd + Unsigned + Display {
    /// Panics if `self` can't turn into a suitable indexing range.
    fn into_index_range(self, end: T) -> Range<T>;
}
impl<T> IntoIndexRange<T> for Range<T> where T: Clone + PartialOrd + Unsigned + Display {
    fn into_index_range(self, end: T) -> Range<T> { check_index_range(self.start .. self.end, end) }
}
impl<T> IntoIndexRange<T> for RangeFrom<T> where T: Clone + PartialOrd + Unsigned + Display {
    fn into_index_range(self, end: T) -> Range<T> { check_index_range(self.start .. end.clone(), end.clone()) }
}
impl<T> IntoIndexRange<T> for RangeTo<T> where T: Clone + PartialOrd + Unsigned + Display {
    fn into_index_range(self, end: T) -> Range<T> { check_index_range(T::zero() .. self.end, end) }
}
impl<T> IntoIndexRange<T> for RangeFull where T: Clone + PartialOrd + Unsigned + Display {
    fn into_index_range(self, end: T) -> Range<T> { check_index_range(T::zero() .. end.clone(), end.clone()) }
}
impl<T> IntoIndexRange<T> for RangeToInclusive<T> where T: Clone + PartialOrd + Unsigned + Display {
    fn into_index_range(self, end: T) -> Range<T> { check_index_range(T::zero() .. self.end+T::one(), end) }
}
impl<T> IntoIndexRange<T> for RangeInclusive<T> where T: Clone + PartialOrd + Unsigned + Display {
    fn into_index_range(self, end: T) -> Range<T> { check_index_range(self.start .. self.end+T::one(), end) }
}
#[cfg(not(debug_assertions))]
#[inline(always)]
#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn check_index_range<T>(index: Range<T>, end: T) -> Range<T> 
    where T: Clone + PartialOrd + Unsigned + Display
{
    index
}
#[cfg(debug_assertions)]
#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn check_index_range<T>(index: Range<T>, end: T) -> Range<T>
    where T: Clone + PartialOrd + Unsigned + Display
{
    if index.start > index.end {
        panic!("Index range starts at {} but ends at {}", index.start, index.end);
    }
    if index.end > end {
        panic!("Index range ends at {} but slice ends at {}", index.end, end);
    }
    index
}

// TODO change this
pub type Row<T> = Vec<T>;

pub trait Many<'a> {
    type Idx: Unsigned;
    type Range = Range<Self::Idx>;
    type One;
    type Ref;
    type RefMut;
    fn with_capacity(cap: Self::Idx) -> Self where Self: Sized;
    fn new() -> Self where Self: Sized {
        Self::with_capacity(Self::Idx::zero())
    }
    fn iter(&'a self) -> <&'a Self as IntoIterator>::IntoIter where &'a Self: IntoIterator {
        self.into_iter()
    }
    fn iter_mut(&'a mut self) -> <&'a mut Self as IntoIterator>::IntoIter where &'a mut Self: IntoIterator {
        self.into_iter()
    }
}

pub trait InsertRow<T> {
    unsafe fn insert_uninitialized(&mut self, index: usize, additional_len: usize);
    fn insert_unchecked(&mut self, index: usize, elem: T);
    fn insert_slice(&mut self, index: usize, elements: &[T]) where T: Clone;
    fn insert_intoiter<I>(&mut self, index: usize, iter: I) where I: IntoIterator<Item=T>;
    fn insert_intoiter_exactsize<I,J>(&mut self, index: usize, iter: I) 
        where I: IntoIterator<Item=T,IntoIter=J>, J: Iterator<Item=T> + ExactSizeIterator;
}
impl<T> InsertRow<T> for Row<T> {
    unsafe fn insert_uninitialized(&mut self, index: usize, additional_len: usize) {
        debug_assert!(index <= self.len());
        let new_len = self.len() + additional_len;
        // The combination of both `reserve()` and `set_len()` has more reasons to be
        // faster than `self.resize(new_len, ::std::mem::uninitialized())`.
        self.reserve(additional_len);
        self.set_len(new_len);
        let src = self.as_ptr().offset(index as isize);
        let dst = (src as *mut T).offset(additional_len as isize);
        // Reminder: using copy_nonoverlapping() would be incorrect here.
        ptr::copy(src, dst, self.len() - additional_len - index);
    }
    #[inline]
    fn insert_unchecked(&mut self, index: usize, elem: T) {
        unsafe {
            self.insert_uninitialized(index, 1);
            *self.get_unchecked_mut(index) = elem;
        }
    }
    fn insert_intoiter<I>(&mut self, index: usize, iter: I) where I: IntoIterator<Item=T> {
        let mut i = index;
        for elem in iter {
            // Instead of Vec's insert() which performs bounds checking.
            self.insert_unchecked(i, elem);
            i += 1;
        }
    }
    fn insert_intoiter_exactsize<I,J>(&mut self, index: usize, iter: I) 
        where I: IntoIterator<Item=T,IntoIter=J>, J: Iterator<Item=T> + ExactSizeIterator
    {
        unsafe {
            let iter = iter.into_iter();
            self.insert_uninitialized(index, iter.len());
            let mut i = 0;
            for elem in iter {
                *self.get_unchecked_mut(index+i) = elem;
                i += 1;
            }
        }
    }
    fn insert_slice(&mut self, index: usize, elements: &[T]) where T: Clone {
        unsafe {
            self.insert_uninitialized(index, elements.len());
            for i in 0..elements.len() {
                *self.get_unchecked_mut(index+i) = elements.get_unchecked(i).clone();
            }
        }
    }
}
