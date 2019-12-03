//! Array with `O(sqrt(N))` arbitrary insertion and deletion.
//! With `O(1)` insertion and deletion from the end of the container
//! and also `O(1)` access like a vector.
//!
//! The contained elements are not required to copyable,
//! and the array will be sendable if the contained type is sendable.
//!
//! An implementation of [**IgushArray**](https://github.com/igushev/IgushArray).

mod util;

use integer_sqrt::IntegerSquareRoot;
use std::cmp::{max, min, Ordering};
use std::fmt;
use std::iter::{repeat_with, FromIterator, FusedIterator};
use std::mem::swap;
use std::ops::{Index, IndexMut, RangeBounds};

const DEFAULT_WIDTH: usize = 32;

/// Array with constant time access and fast insertion and deletion.
/// Compromise in performance between pure array and linked list.
#[derive(Clone)]
pub struct Igush<T> {
    /// backing storage of the structure
    backing: Vec<T>,
    /// where each DEQ is split betwen head and tail
    splits: Vec<usize>,
    /// width of each DEQ
    row_width: usize,
}

impl<T> Igush<T> {
    /// Creates an empty `Igush` with `row_width` elements per row.
    ///
    /// It is recommended to set the `row_width` to approximately `sqrt(N)`
    /// where `N` is the number of elements in the array.
    ///
    /// # Panics
    ///
    /// If `row_width == 0`. `row_width` must be non-zero
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// // for about 100 elements
    /// let array: Igush<u32> = Igush::new(10);
    /// ```
    pub fn new(row_width: usize) -> Self {
        assert!(row_width > 0, "row width must be greater than zero");

        Igush {
            backing: Vec::new(),
            splits: vec![0],
            row_width,
        }
    }

    /// Creates an empty `Igush` with space for at least `capacity` elements.
    ///
    /// It is recommended to set the `row_width` to approximately `sqrt(N)`
    /// where `N` is the number of elements in the array.
    ///
    /// # Panics
    ///
    /// If `row_width == 0`. `row_width` must be non-zero
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let array: Igush<u32> = Igush::with_capacity(10, 100);
    /// ```
    pub fn with_capacity(row_width: usize, capacity: usize) -> Self {
        assert!(row_width > 0, "row width must be greater than zero");

        // always at least one row
        let rows = capacity / row_width + 1;
        let mut splits = Vec::with_capacity(rows);
        splits.push(0);

        Igush {
            backing: Vec::with_capacity(capacity),
            splits,
            row_width,
        }
    }

    /// Retrieves an element in the `Igush` by index.
    ///
    /// Element at index 0 is the front of the array.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut buf = Igush::new(5);
    /// buf.push_back(3);
    /// buf.push_back(4);
    /// buf.push_back(5);
    /// assert_eq!(buf.get(1), Some(&4));
    /// ```
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len() {
            return None;
        }

        let i = self.real_index(index);
        self.backing.get(i)
    }

    /// Retrieves an element in the `Igush` mutably by index.
    ///
    /// Element at index 0 is the front of the array.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut buf = Igush::new(5);
    /// buf.push_back(3);
    /// buf.push_back(4);
    /// buf.push_back(5);
    /// if let Some(elem) = buf.get_mut(1) {
    ///     *elem = 7;
    /// }
    ///
    /// assert_eq!(buf[1], 7);
    /// ```
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len() {
            return None;
        }

        let i = self.real_index(index);
        self.backing.get_mut(i)
    }

    /// Swaps elements at indices `i` and `j`.
    ///
    /// `i` and `j` may be equal.
    ///
    /// Element at index 0 is the front of the array.
    ///
    /// # Panics
    ///
    /// Panics if either index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut buf = Igush::new(5);
    /// buf.push_back(3);
    /// buf.push_back(4);
    /// buf.push_back(5);
    /// assert_eq!(buf, [3, 4, 5]);
    /// buf.swap(0, 2);
    /// assert_eq!(buf, [5, 4, 3]);
    /// ```
    pub fn swap(&mut self, i: usize, j: usize) {
        assert!(i < self.len(), "index out of bounds");
        assert!(j < self.len(), "index out of bounds");

        let real_i = self.real_index(i);
        let real_j = self.real_index(j);

        self.backing.swap(real_i, real_j);
    }

    /// Returns the number of elements the `Igush` can hold without
    /// reallocating.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let buf: Igush<i32> = Igush::with_capacity(5, 10);
    /// assert!(buf.capacity() >= 10);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.backing.capacity()
    }

    /// Reserves the minimum capacity for exactly `additional` more elements to be inserted in the
    /// given `Igush`. Does nothing if the capacity is already sufficient.
    ///
    /// Note that the allocator may give the collection more space than it requests. Therefore
    /// capacity can not be relied upon to be precisely minimal. Prefer [`reserve`] if future
    /// insertions are expected.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity overflows `usize`.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut buf: Igush<i32> = vec![1].into();
    /// buf.reserve_exact(10);
    /// assert!(buf.capacity() >= 11);
    /// ```
    ///
    /// [`reserve`]: #method.reserve
    pub fn reserve_exact(&mut self, additional: usize) {
        self.backing.reserve_exact(additional);
    }

    /// Reserves capacity for at least `additional` more elements to be inserted in the given
    /// `Igush`. The collection may reserve more space to avoid frequent reallocations.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity overflows `usize`.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut buf: Igush<i32> = vec![1].into();
    /// buf.reserve(10);
    /// assert!(buf.capacity() >= 11);
    /// ```
    pub fn reserve(&mut self, additional: usize) {
        self.backing.reserve(additional);
    }

    /// Shrinks the capacity of the `Igush` as much as possible.
    ///
    /// It will drop down as close as possible to the length but the allocator may still inform the
    /// `Igush` that there is space for a few more elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut buf = Igush::with_capacity(5, 15);
    /// buf.extend_back(0..4);
    /// assert_eq!(buf.capacity(), 15);
    /// buf.shrink_to_fit();
    /// assert!(buf.capacity() >= 4);
    /// ```
    pub fn shrink_to_fit(&mut self) {
        self.backing.shrink_to_fit();
    }

    /// Shortens the `Igush`, dropping excess elements from the back.
    ///
    /// If `len` is greater than the `Igush`'s current length, this has no
    /// effect.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut buf = Igush::new(5);
    /// buf.push_back(5);
    /// buf.push_back(10);
    /// buf.push_back(15);
    /// assert_eq!(buf, [5, 10, 15]);
    /// buf.truncate(1);
    /// assert_eq!(buf, [5]);
    /// ```
    pub fn truncate(&mut self, len: usize) {
        for _ in len..self.len() {
            self.pop_back();
        }
    }

    /// Returns a front-to-back iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut buf = Igush::new(5);
    /// buf.push_back(5);
    /// buf.push_back(3);
    /// buf.push_back(4);
    /// let c: Vec<&i32> = buf.iter().collect();
    /// assert_eq!(c, vec![&5, &3, &4]);
    /// ```
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = &T> + DoubleEndedIterator + ExactSizeIterator + FusedIterator {
        (0..self.len())
            .into_iter()
            .map(move |index| self.get(index).unwrap())
    }

    /// Returns a front-to-back iterator that returns mutable references.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut buf = Igush::new(5);
    /// buf.push_back(5);
    /// buf.push_back(3);
    /// buf.push_back(4);
    /// for num in buf.iter_mut() {
    ///     *num = *num - 2;
    /// }
    /// let c: Vec<&mut i32> = buf.iter_mut().collect();
    /// assert_eq!(c, vec![&mut 3, &mut 1, &mut 2]);
    /// ```
    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut T> + DoubleEndedIterator + ExactSizeIterator + FusedIterator
    {
        (0..self.len()).into_iter().map(move |index| {
            // should be possible to just do this
            // self.get_mut(index).unwrap()

            // FIXME: horrible hack
            // can't figure out any other way of doing this
            // but I think it's sound
            // seems accessing self.splits immutably
            // and getting a mutable reference to self.backing
            // trips up the borrow checker
            let item = self.get(index).unwrap();
            unsafe { &mut *(item as *const T as *mut T) }
        })
    }

    // choosing not to implement as_slices, as_mut_slices
    // they don't make sense

    /// Returns the number of elements in the `Igush`.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut v = Igush::new(5);
    /// assert_eq!(v.len(), 0);
    /// v.push_back(1);
    /// assert_eq!(v.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.backing.len()
    }

    /// Returns `true` if the `Igush` is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut v = Igush::new(5);
    /// assert!(v.is_empty());
    /// v.push_front(1);
    /// assert!(!v.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Creates a draining iterator that removes the specified range in the
    /// `Igush` and yields the removed items.
    ///
    /// Note: The element range is removed even if the iterator is not
    /// consumed until the end.
    ///
    /// # Panics
    ///
    /// Panics if the starting point is greater than the end point or if
    /// the end point is greater than the length of the vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut v: Igush<_> = vec![1, 2, 3].into();
    /// let mut drained = Igush::new(5);
    /// drained.extend_back(v.drain(2..));
    /// assert_eq!(drained, [3]);
    /// assert_eq!(v, [1, 2]);
    ///
    /// // A full range clears all contents
    /// v.drain(..);
    /// assert!(v.is_empty());
    /// ```
    #[inline]
    pub fn drain<R>(
        &mut self,
        range: R,
    ) -> impl Iterator<Item = T> + DoubleEndedIterator + ExactSizeIterator + FusedIterator
    where
        R: RangeBounds<usize>,
    {
        use std::ops::Bound::*;

        let len = self.len();
        let start = match range.start_bound() {
            Included(&n) => n,
            Excluded(&n) => n + 1,
            Unbounded => 0,
        };
        let end = match range.end_bound() {
            Included(&n) => n + 1,
            Excluded(&n) => n,
            Unbounded => len,
        };
        assert!(start <= end, "drain lower bound was too large");
        assert!(end <= len, "drain upper bound was too large");

        let out: Vec<T> = (start..end)
            .rev()
            .map(|index| self.remove(index).unwrap())
            .collect();
        out.into_iter().rev()
    }

    /// Clears the `Igush`, removing all values.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut v = Igush::new(5);
    /// v.push_back(1);
    /// v.clear();
    /// assert!(v.is_empty());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.splits.clear();
        self.splits.push(0);
        self.backing.clear();
    }

    /// Returns `true` if the `Igush` contains an element equal to the
    /// given value.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut vector: Igush<u32> = Igush::new(5);
    ///
    /// vector.push_back(0);
    /// vector.push_back(1);
    ///
    /// assert_eq!(vector.contains(&1), true);
    /// assert_eq!(vector.contains(&10), false);
    /// ```
    pub fn contains(&self, x: &T) -> bool
    where
        T: PartialEq<T>,
    {
        self.backing.contains(x)
    }

    /// Provides a reference to the front element, or `None` if the `Igush` is
    /// empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut d = Igush::new(5);
    /// assert_eq!(d.front(), None);
    ///
    /// d.push_back(1);
    /// d.push_back(2);
    /// assert_eq!(d.front(), Some(&1));
    /// ```
    pub fn front(&self) -> Option<&T> {
        self.get(0)
    }

    /// Provides a mutable reference to the front element, or `None` if the
    /// `Igush` is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut d = Igush::new(5);
    /// assert_eq!(d.front_mut(), None);
    ///
    /// d.push_back(1);
    /// d.push_back(2);
    /// match d.front_mut() {
    ///     Some(x) => *x = 9,
    ///     None => (),
    /// }
    /// assert_eq!(d.front(), Some(&9));
    /// ```
    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.get_mut(0)
    }

    /// Provides a reference to the back element, or `None` if the `Igush` is
    /// empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut d = Igush::new(5);
    /// assert_eq!(d.back(), None);
    ///
    /// d.push_back(1);
    /// d.push_back(2);
    /// assert_eq!(d.back(), Some(&2));
    /// ```
    pub fn back(&self) -> Option<&T> {
        self.backing.last()
    }

    /// Provides a mutable reference to the back element, or `None` if the
    /// `Igush` is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut d = Igush::new(5);
    /// assert_eq!(d.back(), None);
    ///
    /// d.push_back(1);
    /// d.push_back(2);
    /// match d.back_mut() {
    ///     Some(x) => *x = 9,
    ///     None => (),
    /// }
    /// assert_eq!(d.back(), Some(&9));
    /// ```
    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.backing.last_mut()
    }

    /// Removes the first element and returns it, or `None` if the `Igush` is
    /// empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut d = Igush::new(5);
    /// d.push_back(1);
    /// d.push_back(2);
    ///
    /// assert_eq!(d.pop_front(), Some(1));
    /// assert_eq!(d.pop_front(), Some(2));
    /// assert_eq!(d.pop_front(), None);
    /// ```
    pub fn pop_front(&mut self) -> Option<T> {
        if let Some(last_row) = self.last_row() {
            // remove first element from last row
            // last DEQ is always kept contiguous
            let mut temp = self.backing.remove(last_row * self.row_width);

            // iterate through full rows, swapping out first element
            for row in (0..last_row).rev() {
                let split = self.splits[row];
                let start = (row * self.row_width) + split;

                // swap into previous row
                swap(&mut self.backing[start], &mut temp);
                // move split
                self.splits[row] = util::wrap_add(self.row_width, split, 1);
            }

            self.correct_splits();

            Some(temp)
        } else {
            None
        }
    }

    /// Removes the last element from the `Igush` and returns it, or `None` if
    /// it is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut buf = Igush::new(5);
    /// assert_eq!(buf.pop_back(), None);
    /// buf.push_back(1);
    /// buf.push_back(3);
    /// assert_eq!(buf.pop_back(), Some(3));
    /// ```
    pub fn pop_back(&mut self) -> Option<T> {
        // last DEQ is always kept contiguous
        // so popping is just popping from the Vec
        let element = self.backing.pop();
        self.correct_splits();

        element
    }

    /// Prepends an element to the `Igush`.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut d = Igush::new(5);
    /// d.push_front(1);
    /// d.push_front(2);
    /// assert_eq!(d.front(), Some(&2));
    /// ```
    pub fn push_front(&mut self, element: T) {
        let mut temp = element;
        // iterate through full rows, swapping out last element
        for row in 0..self.back_row() {
            let split = self.splits[row];
            let last = (row * self.row_width) + util::wrap_add(self.row_width, split, -1);

            // swap into next row
            swap(&mut self.backing[last], &mut temp);
            // move split
            self.splits[row] = util::wrap_add(self.row_width, split, -1);
        }

        // insert at beginning of last row to maintain contiguity
        self.backing.insert(self.back_row() * self.row_width, temp);

        self.correct_splits();
    }

    /// Appends an element to the back of the `Igush`.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut buf = Igush::new(5);
    /// buf.push_back(1);
    /// buf.push_back(3);
    /// assert_eq!(3, *buf.back().unwrap());
    /// ```
    pub fn push_back(&mut self, element: T) {
        // last DEQ is always kept contiguous
        // so pushing is just pushing to the Vec
        self.backing.push(element);

        self.correct_splits();
    }

    // choosing to not implement `swap_remove_front` or `swap_remove_back`
    // because ordering is kinda the whole point

    /// Inserts an element at `index` within the `Igush`, shifting all elements with indices
    /// greater than or equal to `index` towards the back.
    ///
    /// Element at index 0 is the front of the queue.
    ///
    /// # Panics
    ///
    /// Panics if `index` is greater than `Igush`'s length
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut buf = Igush::new(5);
    /// buf.push_back('a');
    /// buf.push_back('b');
    /// buf.push_back('c');
    /// assert_eq!(buf, ['a', 'b', 'c']);
    ///
    /// buf.insert(1, 'd');
    /// assert_eq!(buf, ['a', 'd', 'b', 'c']);
    /// ```
    pub fn insert(&mut self, index: usize, element: T) {
        assert!(index <= self.len(), "index out of bounds");

        let (target_row, target_col) = util::div_rem(index, self.row_width);

        if target_row == self.back_row() {
            self.backing.insert(index, element);

            self.correct_splits();
            return;
        }

        let mut temp = element;
        {
            let split = self.splits[target_row];
            let start = target_row * self.row_width;
            let end = start + self.row_width;
            let row = &mut self.backing[start..end];
            let last = util::wrap_add(self.row_width, split, -1);

            // swap in new element
            swap(&mut row[last], &mut temp);

            // rotate new element into position

            // before [ i x|e f g h ]
            // after  [ i|e f g x h ]
            //
            //        [ i x|e f g h ]
            //           {       } rotate_left(1)

            // before [ g h i x|e f ]
            // after  [ g x h i|e f ]
            //
            //        [ g h i x|e f ]
            //           {     } rotate_right(1)

            // before [ i x|e f g h ]
            // after  [ i|e x f g h ]
            //
            //        [ i x|e f g h ]
            //           {   } rotate_left(1)

            // before [ g h i x|e f ]
            // after  [ g h i|e f x ]
            //
            //        [ g h i x|e f ]
            //               {     } rotate_left(1)

            // if before position < after position,
            //   [before..after].rotate_left(1)
            //   shift split left
            // if before position > after position,
            //   [after..=before].rotate_right(1)
            let before = last;
            let after = util::wrap_add(self.row_width, split, target_col as isize);

            if before <= after {
                row[before..=after].rotate_left(1);
                self.splits[target_row] = util::wrap_add(self.row_width, split, -1);
            } else {
                row[after..=before].rotate_right(1);
            }
        }

        // iterate through remaining full rows, swapping out last element
        for row in (target_row + 1)..self.back_row() {
            let split = self.splits[row];
            let last = (row * self.row_width) + util::wrap_add(self.row_width, split, -1);

            // swap into next row
            swap(&mut self.backing[last], &mut temp);
            // move split
            self.splits[row] = util::wrap_add(self.row_width, split, -1);
        }

        // insert at beginning of last row to maintain contiguity
        self.backing.insert(self.back_row() * self.row_width, temp);

        self.correct_splits();
    }

    /// Removes and returns the element at `index` from the `Igush`.
    /// Whichever end is closer to the removal point will be moved to make
    /// room, and all the affected elements will be moved to new positions.
    /// Returns `None` if `index` is out of bounds.
    ///
    /// Element at index 0 is the front of the queue.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut buf = Igush::new(5);
    /// buf.push_back(1);
    /// buf.push_back(2);
    /// buf.push_back(3);
    /// assert_eq!(buf, [1, 2, 3]);
    ///
    /// assert_eq!(buf.remove(1), Some(2));
    /// assert_eq!(buf, [1, 3]);
    /// ```
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len() {
            return None;
        }

        let last_row = self.last_row().unwrap();
        let (target_row, target_col) = util::div_rem(index, self.row_width);

        if target_row == last_row {
            let element = self.backing.remove(index);
            self.correct_splits();

            return Some(element);
        }

        // remove first element from last row
        // last DEQ is always kept contiguous
        let mut temp = self.backing.remove(last_row * self.row_width);

        // iterate through full rows, swapping out first element
        for row in ((target_row + 1)..last_row).rev() {
            let split = self.splits[row];
            let start = (row * self.row_width) + split;

            // swap into previous row
            swap(&mut self.backing[start], &mut temp);
            // move split
            self.splits[row] = util::wrap_add(self.row_width, split, 1);
        }

        {
            let split = self.splits[target_row];
            let start = target_row * self.row_width;
            let end = start + self.row_width;
            let row = &mut self.backing[start..end];
            let col = util::wrap_add(self.row_width, split, target_col as isize);

            // swap into target row
            swap(&mut row[col], &mut temp);

            // rotate new element into position

            // before [ i|e f g x h ]
            // after  [ i x|e f g h ]
            //
            //        [ i|e f g x h ]
            //           {       } rotate_right(1)

            // before [ g x h i|e f ]
            // after  [ g h i x|e f ]
            //
            //        [ g x h i|e f ]
            //           {     } rotate_left(1)

            // before [ i|e x f g h ]
            // after  [ i x|e f g h ]
            //
            //        [ i|e x f g h ]
            //           {   } rotate_right(1)

            // before [ i|x e f g h ]
            // after  [ i x|e f g h ]
            //
            //        [ i x|e f g h ]
            //           { } rotate_right(1)

            // before [ g h i|e f x ]
            // after  [ g h i x|e f ]
            //
            //        [ g h i|e f x ]
            //               {     } rotate_right(1)

            // if before position < after position,
            //   [before..=after].rotate_left(1)
            // if before position >= after position,
            //   [after..=before].rotate_right(1)
            //   shift split right
            let before = col;
            let after = split;

            if before < after {
                row[before..=after].rotate_left(1);
            } else {
                row[after..=before].rotate_right(1);
                self.splits[target_row] = util::wrap_add(self.row_width, split, 1);
            }
        }

        self.correct_splits();

        Some(temp)
    }

    /// Splits the `Igush` into two at the given index.
    ///
    /// Returns a newly allocated `Igush`. `self` contains elements `[0, at)`,
    /// and the returned `Igush` contains elements `[at, len)`.
    ///
    /// - Note 1: the capacity of `self` does not change.
    /// - Note 2: may result in O(n) data movement.
    ///
    /// Element at index 0 is the front of the queue.
    ///
    /// # Panics
    ///
    /// Panics if `at > len`.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut buf: Igush<_> = vec![1,2,3].into_iter().collect();
    /// let buf2 = buf.split_off(1);
    /// assert_eq!(buf, [1]);
    /// assert_eq!(buf2, [2, 3]);
    /// ```
    #[inline]
    pub fn split_off(&mut self, at: usize) -> Self {
        let len = self.len();
        assert!(at <= len, "`at` out of bounds");

        self.make_contiguous();
        let other = self.backing.split_off(at);

        self.correct_splits();
        other.into()
    }

    /// Moves all the elements of `other` into `self`, leaving `other` empty.
    ///
    /// # Panics
    ///
    /// Panics if the new number of elements in self overflows a `usize`.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut buf: Igush<_> = vec![1, 2].into();
    /// let mut buf2: Igush<_> = vec![3, 4].into();
    /// buf.append(&mut buf2);
    /// assert_eq!(buf, [1, 2, 3, 4]);
    /// assert_eq!(buf2, []);
    /// ```
    #[inline]
    pub fn append(&mut self, other: &mut Self) {
        // naive impl
        self.extend_back(other.drain(..));
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements `e` such that `f(&e)` returns false.
    /// This method operates in place, visiting each element exactly once in the
    /// original order, and preserves the order of the retained elements.
    ///
    /// Note: may result in O(n) data movement.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut buf = Igush::new(5);
    /// buf.extend_back(1..5);
    /// buf.retain(|&x| x % 2 == 0);
    /// assert_eq!(buf, [2, 4]);
    /// ```
    ///
    /// The exact order may be useful for tracking external state, like an index.
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut buf = Igush::new(5);
    /// buf.extend_back(1..6);
    ///
    /// let keep = [false, true, true, false, true];
    /// let mut i = 0;
    /// buf.retain(|_| (keep[i], i += 1).0);
    /// assert_eq!(buf, [2, 3, 5]);
    /// ```
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        // naive impl
        self.make_contiguous();
        self.backing.retain(f);
    }

    /// Modifies the `Igush` in-place so that `len()` is equal to `new_len`,
    /// either by removing excess elements from the back or by appending
    /// elements generated by calling `generator` to the back.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut buf = Igush::new(5);
    /// buf.push_back(5);
    /// buf.push_back(10);
    /// buf.push_back(15);
    /// assert_eq!(buf, [5, 10, 15]);
    ///
    /// buf.resize_with(5, Default::default);
    /// assert_eq!(buf, [5, 10, 15, 0, 0]);
    ///
    /// buf.resize_with(2, || unreachable!());
    /// assert_eq!(buf, [5, 10]);
    ///
    /// let mut state = 100;
    /// buf.resize_with(5, || { state += 1; state });
    /// assert_eq!(buf, [5, 10, 101, 102, 103]);
    /// ```
    pub fn resize_with(&mut self, new_len: usize, generator: impl FnMut() -> T) {
        let len = self.len();

        if new_len > len {
            self.extend_back(repeat_with(generator).take(new_len - len))
        } else {
            self.truncate(new_len);
        }
    }

    /// Modifies the `Igush` in-place so that `len()` is equal to new_len,
    /// either by removing excess elements from the back or by appending clones of `value`
    /// to the back.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// let mut buf = Igush::new(5);
    /// buf.push_back(5);
    /// buf.push_back(10);
    /// buf.push_back(15);
    /// assert_eq!(buf, [5, 10, 15]);
    ///
    /// buf.resize(2, 0);
    /// assert_eq!(buf, [5, 10]);
    ///
    /// buf.resize(5, 20);
    /// assert_eq!(buf, [5, 10, 20, 20, 20]);
    /// ```
    pub fn resize(&mut self, new_len: usize, value: T)
    where
        T: Clone,
    {
        self.resize_with(new_len, || value.clone());
    }

    // TODO: rotate_left, rotate_right

    /// first non-full row index
    fn back_row(&self) -> usize {
        self.len() / self.row_width
    }

    /// last non-empty row index
    fn last_row(&self) -> Option<usize> {
        if self.is_empty() {
            return None;
        }

        let (quo, rem) = util::div_rem(self.len(), self.row_width);
        if rem == 0 {
            Some(quo - 1)
        } else {
            Some(quo)
        }
    }

    /// remove or add elements to `self.splits`
    /// to match number of rows
    fn correct_splits(&mut self) {
        // always at least one row
        let rows = (self.len() / self.row_width) + 1;
        while self.splits.len() < rows {
            self.splits.push(0);
        }
        while self.splits.len() > rows {
            debug_assert_eq!(self.splits.pop(), Some(0), "removed row was not contiguous");
        }

        debug_assert!(self.splits.len() > 0);

        // ensure last row is contiguous
        // debug_assert_eq!(self.splits.last().unwrap(), 0);

        // make last row contiguous
        let last = self.splits.len() - 1;
        let split = self.splits[last];
        if split != 0 {
            let start = last * self.row_width;
            let end = min(start + self.row_width, self.len());
            // dbg!(last, self.len(), start, end);
            let row = &mut self.backing[start..end];

            util::make_contiguous(row, split);
            self.splits[last] = 0;
        }
    }

    /// calculate the real index in `backing` for a given index
    fn real_index(&self, index: usize) -> usize {
        let (target_row, target_col) = util::div_rem(index, self.row_width);

        let real_col = util::wrap_add(self.row_width, self.splits[target_row], target_col as isize);
        (target_row * self.row_width) + real_col
    }

    /// Appends items from the contents of the iterator.
    pub fn extend_back<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for elem in iter {
            self.push_back(elem);
        }
    }

    /// Prepend items from the contents of the iterator.
    pub fn extend_front<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for elem in iter {
            self.push_front(elem);
        }
    }

    /// Make the backing stucture completely contiguous.
    ///
    /// Returns a reference to the _now-contiguous_ backing vector.
    pub fn make_contiguous(&mut self) -> &mut Vec<T> {
        if let Some(last_row) = self.last_row() {
            let row_width = self.row_width;

            // make all rows contiguous
            // return base Vec
            for i in 0..=last_row {
                let start = i * row_width;
                let end = min(start + row_width, self.backing.len());
                let row = &mut self.backing[start..end];

                let split = self.splits[i];
                if split > 0 {
                    util::make_contiguous(row, split);
                }
            }
        }

        &mut self.backing
    }
}

impl<T: PartialEq> PartialEq for Igush<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter().eq(other.iter())
    }
}
impl<T: Eq> Eq for Igush<T> {}

macro_rules! __impl_slice_eq {
    ($lhs:ty, $rhs:ty) => {
        impl<A, B> PartialEq<$rhs> for $lhs
        where
            A: PartialEq<B>,
        {
            fn eq(&self, other: &$rhs) -> bool {
                if self.len() != other.len() {
                    return false;
                }

                self.iter().eq(other.iter())
            }
        }
    };
}

__impl_slice_eq! { Igush<A>, Vec<B> }
__impl_slice_eq! { Igush<A>, &[B] }
__impl_slice_eq! { Igush<A>, &mut [B] }

macro_rules! __impl_array_eq {
    ( $( $n:tt ),+ ) => {
        $(
        __impl_slice_eq! { Igush<A>, [B; $n] }
        __impl_slice_eq! { Igush<A>, &[B; $n] }
        __impl_slice_eq! { Igush<A>, &mut [B; $n] }
        )*
    }
}

// implement equality for arrays up to length 32
// FIXME: implement with const generics eventually
__impl_array_eq![
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32
];

impl<T: PartialOrd> PartialOrd for Igush<T> {
    fn partial_cmp(&self, other: &Igush<T>) -> Option<Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

impl<T: Ord> Ord for Igush<T> {
    #[inline]
    fn cmp(&self, other: &Igush<T>) -> Ordering {
        self.iter().cmp(other.iter())
    }
}

use std::hash::{Hash, Hasher};

impl<T: Hash> Hash for Igush<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // FIXME: naive impl
        self.iter().collect::<Vec<_>>().hash(state);
    }
}

impl<T> Index<usize> for Igush<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("Out of bounds access")
    }
}
impl<T> IndexMut<usize> for Igush<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).expect("Out of bounds access")
    }
}

impl<T> FromIterator<T> for Igush<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let iterator = iter.into_iter();
        let (lower, _) = iterator.size_hint();
        let row_width = max(lower.integer_sqrt(), DEFAULT_WIDTH);
        let mut deq = Igush::with_capacity(row_width, lower);
        deq.extend_back(iterator);
        deq
    }
}

impl<T> IntoIterator for Igush<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    /// Consumes the `Igush` into a front-to-back iterator yielding elements by value.
    ///
    /// Note: may result in O(n) data movement.
    fn into_iter(mut self) -> Self::IntoIter {
        self.make_contiguous();
        self.backing.into_iter()
    }
}

// TODO: impl IntoIterator<&Igush<T>> and IntoIterator<&mut Igush<T>>

// Currently not possible
// `impl Trait` in type aliases is unstable
// for more information, see https://github.com/rust-lang/rust/issues/63063
//
// impl<T> IntoIterator for &Igush<T> {
//     type Item = T;
//     type IntoIter = impl Iterator<Item = &T> + DoubleEndedIterator + ExactSizeIterator + FusedIterator;
//
//     fn into_iter(self) -> Self::IntoIter {
//         self.iter()
//     }
// }
//
// impl<T> IntoIterator for &mut Igush<T> {
//     type Item = T;
//     type IntoIter = impl Iterator<Item = &mut T> + DoubleEndedIterator + ExactSizeIterator + FusedIterator;
//
//     fn into_iter(self) -> Self::IntoIter {
//         self.iter_mut()
//     }
// }

impl<T> Extend<T> for Igush<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.extend_back(iter);
    }
}

impl<T: fmt::Debug> fmt::Debug for Igush<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T> From<Vec<T>> for Igush<T> {
    /// Turn a `Vec<T>` into an `Igush<T>`.
    ///
    /// Automatically choses `row_width` as `sqrt(N)`
    /// where `N` is the length of the `Vec`.
    ///
    /// This never needs to reallocate.
    fn from(other: Vec<T>) -> Self {
        let row_width = max(other.len().integer_sqrt(), DEFAULT_WIDTH);
        // always at least one row
        let rows = other.len() / row_width + 1;

        Igush {
            backing: other,
            splits: vec![0; rows],
            row_width,
        }
    }
}

impl<'a, T: Clone> From<&'a [T]> for Igush<T> {
    /// Creates an `Igush<T>` from a slice.
    ///
    /// Automatically choses `row_width` as `sqrt(N)`
    /// where `N` is the length of the slice.
    fn from(other: &[T]) -> Self {
        let row_width = max(other.len().integer_sqrt(), DEFAULT_WIDTH);
        // always at least one row
        let rows = other.len() / row_width + 1;

        Igush {
            backing: other.into(),
            splits: vec![0; rows],
            row_width,
        }
    }
}

/// Create a new `Igush` instance.
/// Like the `vec![]` macro
///
/// ```
/// use igush_rs::{Igush, igush};
///
/// let array = igush![1, 2, 3, 4];
/// assert_eq!(array.len(), 4);
/// 
/// let array = igush![5; 10];
/// assert_eq!(array.len(), 10);
/// 
/// let array = igush![
///     1, 2, 3, 4,
///     5, 6, 7, 8,
/// ];
/// assert_eq!(array.len(), 8);
/// ```
#[macro_export]
macro_rules! igush {
    ($elem:expr; $n:expr) => (
        Igush::from(vec![$elem; $n])
    );
    ($($x:expr),*) => (
        Igush::from(vec![$($x),*])
    );
    ($($x:expr,)*) => (
        Igush::from(vec![ $($x),* ])
    );
}

impl<T> Into<Vec<T>> for Igush<T> {
    /// Turn a `Igush<T>` into a `Vec<T>`.
    ///
    /// This never needs to re-allocate, but does need to do O(n) data movement
    /// when the backing vector is not contiguous.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::Igush;
    ///
    /// // This one is O(1).
    /// let deque: Igush<_> = (1..5).collect();
    /// let vec: Vec<_> = deque.into();
    /// assert_eq!(vec, [1, 2, 3, 4]);
    ///
    /// // This one needs data rearranging.
    /// let mut deque: Igush<_> = (1..5).collect();
    /// deque.push_front(9);
    /// deque.push_front(8);
    /// let vec: Vec<_> = deque.into();
    /// assert_eq!(vec, [8, 9, 1, 2, 3, 4]);
    /// ```
    fn into(mut self) -> Vec<T> {
        self.make_contiguous();
        self.backing
    }
}

/// Tries to implement applicable methods from `Vec<T>` and `[T]`
///
/// Excludes most ordered methods like the following because
/// they don't make sense on a non-contiguous array.
///
/// - stable `sort*`
/// - `starts_with`, `ends_with`
/// - `*chunks*`
/// - `*split*`
/// - `windows`
/// - `reverse`
/// - `binary_search*`
/// - `rotate*`
/// - etc
///
/// To use them, call [`make_contiguous`] to get a mutable reference
/// to a contiguous underlying `Vec`.
///
/// [`make_contiguous`]: struct.Igush.html#method.make_contiguous
pub trait VecCompat<T> {
    fn push(&mut self, element: T);
    fn pop(&mut self) -> Option<T>;
    // fn remove(&mut self, index: usize) -> T;
    fn first(&self) -> Option<&T>;
    fn first_mut(&mut self) -> Option<&mut T>;
    fn last(&self) -> Option<&T>;
    fn last_mut(&mut self) -> Option<&mut T>;
    fn sort_unstable(&mut self)
    where
        T: Ord;
    fn sort_unstable_by<F>(&mut self, compare: F)
    where
        T: Ord,
        F: FnMut(&T, &T) -> Ordering;
    fn sort_unstable_by_key<K, F>(&mut self, f: F)
    where
        T: Ord,
        F: FnMut(&T) -> K,
        K: Ord;
}

impl<T> VecCompat<T> for Igush<T> {
    /// Appends an element to the back of a collection.
    ///
    /// # Panics
    ///
    /// Panics if the number of elements in the vector overflows a `usize`.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::{Igush, igush, VecCompat};
    ///
    /// let mut vec = igush![1, 2];
    /// vec.push(3);
    /// assert_eq!(vec, [1, 2, 3]);
    /// ```
    #[inline]
    fn push(&mut self, element: T) {
        self.push_back(element);
    }

    /// Removes the last element from a vector and returns it, or `None` if it
    /// is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::{Igush, igush, VecCompat};
    ///
    /// let mut vec = igush![1, 2, 3];
    /// assert_eq!(vec.pop(), Some(3));
    /// assert_eq!(vec, [1, 2]);
    /// ```
    #[inline]
    fn pop(&mut self) -> Option<T> {
        self.pop_back()
    }

    // /// Removes and returns the element at position `index` within the vector,
    // /// shifting all elements after it to the left.
    // ///
    // /// # Panics
    // ///
    // /// Panics if `index` is out of bounds.
    // ///
    // /// # Examples
    // ///
    // /// ```
    // /// use igush_rs::{Igush, igush, VecCompat};
    // /// 
    // /// let mut v = igush![1, 2, 3];
    // /// assert_eq!(v.remove(1), 2);
    // /// assert_eq!(v, [1, 3]);
    // /// ```
    // fn remove(&mut self, index: usize) -> T {
    //     self.remove(index).expect("index out of bounds")
    // }

    /// Returns the first element of the slice, or `None` if it is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::{Igush, igush, VecCompat};
    ///
    /// let v = igush![10, 40, 30];
    /// assert_eq!(Some(&10), v.first());
    ///
    /// let w: Igush<i32> = igush![];
    /// assert_eq!(None, w.first());
    /// ```
    #[inline]
    fn first(&self) -> Option<&T> {
        self.front()
    }

    /// Returns a mutable pointer to the first element of the slice, or `None` if it is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::{Igush, igush, VecCompat};
    ///
    /// let mut v = igush![0, 1, 2];
    /// let x = &mut v;
    ///
    /// if let Some(first) = x.first_mut() {
    ///     *first = 5;
    /// }
    /// assert_eq!(x, &[5, 1, 2]);
    /// ```
    #[inline]
    fn first_mut(&mut self) -> Option<&mut T> {
        self.front_mut()
    }

    /// Returns the last element of the slice, or `None` if it is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::{Igush, igush, VecCompat};
    ///
    /// let v = igush![10, 40, 30];
    /// assert_eq!(Some(&30), v.last());
    ///
    /// let w: Igush<i32> = igush![];
    /// assert_eq!(None, w.last());
    /// ```
    #[inline]
    fn last(&self) -> Option<&T> {
        self.back()
    }

    /// Returns a mutable pointer to the last item in the slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::{Igush, igush, VecCompat};
    ///
    /// let mut v = igush![0, 1, 2];
    /// let x = &mut v;
    ///
    /// if let Some(last) = x.last_mut() {
    ///     *last = 10;
    /// }
    /// assert_eq!(x, &[0, 1, 10]);
    /// ```
    #[inline]
    fn last_mut(&mut self) -> Option<&mut T> {
        self.back_mut()
    }

    /// Sorts the slice, but may not preserve the order of equal elements.
    ///
    /// This sort is unstable (i.e., may reorder equal elements), in-place
    /// (i.e., does not allocate), and `O(n log n)` worst-case.
    ///
    /// See [Vec#sort_unstable] for more.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::{Igush, igush, VecCompat};
    ///
    /// let mut v = igush![-5, 4, 1, -3, 2];
    ///
    /// v.sort_unstable();
    /// assert!(v == [-5, -3, 1, 2, 4]);
    /// ```
    ///
    /// [Vec#sort_unstable]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.sort_unstable
    #[inline]
    fn sort_unstable(&mut self)
    where
        T: Ord,
    {
        self.backing.sort_unstable();
        self.splits.iter_mut().for_each(|x| *x = 0);
    }

    /// Sorts the slice with a comparator function, but may not preserve the order of equal
    /// elements.
    ///
    /// This sort is unstable (i.e., may reorder equal elements), in-place
    /// (i.e., does not allocate), and `O(n log n)` worst-case.
    ///
    /// See [Vec#sort_unstable_by] for more.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::{Igush, igush, VecCompat};
    ///
    /// let mut v = igush![5, 4, 1, 3, 2];
    /// v.sort_unstable_by(|a, b| a.cmp(b));
    /// assert!(v == [1, 2, 3, 4, 5]);
    ///
    /// // reverse sorting
    /// v.sort_unstable_by(|a, b| b.cmp(a));
    /// assert!(v == [5, 4, 3, 2, 1]);
    /// ```
    ///
    /// [Vec#sort_unstable_by]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.sort_unstable_by
    #[inline]
    fn sort_unstable_by<F>(&mut self, compare: F)
    where
        T: Ord,
        F: FnMut(&T, &T) -> Ordering,
    {
        self.backing.sort_unstable_by(compare);
        self.splits.iter_mut().for_each(|x| *x = 0);
    }

    /// Sorts the slice with a key extraction function, but may not preserve the order of equal
    /// elements.
    ///
    /// This sort is unstable (i.e., may reorder equal elements), in-place
    /// (i.e., does not allocate), and `O(m n log(m n))` worst-case, where the key function is
    /// `O(m)`.
    ///
    /// See [Vec#sort_unstable_by_key] for more.
    ///
    /// # Examples
    ///
    /// ```
    /// use igush_rs::{Igush, igush, VecCompat};
    ///
    /// let mut v = igush![-5i32, 4, 1, -3, 2];
    ///
    /// v.sort_unstable_by_key(|k| k.abs());
    /// assert!(v == [1, 2, -3, 4, -5]);
    /// ```
    ///
    /// [Vec#sort_unstable_by_key]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.sort_unstable_by
    #[inline]
    fn sort_unstable_by_key<K, F>(&mut self, f: F)
    where
        T: Ord,
        F: FnMut(&T) -> K,
        K: Ord,
    {
        self.backing.sort_unstable_by_key(f);
        self.splits.iter_mut().for_each(|x| *x = 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let created: Igush<i32> = Igush::with_capacity(32, 0);

        assert_eq!(created.capacity(), 0);
        assert_eq!(created.len(), 0);

        let created: Igush<i32> = Igush::with_capacity(5, 5);

        assert_eq!(created.capacity(), 5);
        assert_eq!(created.len(), 0);

        let created: Igush<i32> = Igush::with_capacity(10, 11);

        assert_eq!(created.capacity(), 11);
        assert_eq!(created.len(), 0);
    }

    #[test]
    fn push_back() {
        let mut array: Igush<i32> = Igush::new(5);

        for i in 0..20 {
            array.push_back(i);
        }

        assert_eq!(array.len(), 20);
        assert_eq!(
            array,
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]
        );
    }

    #[test]
    fn push_front() {
        let mut array: Igush<i32> = Igush::new(5);

        for i in 0..20 {
            array.push_front(i);
        }

        assert_eq!(array.len(), 20);
        assert_eq!(
            array,
            [19, 18, 17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0]
        );
    }

    #[test]
    fn get() {
        let mut array: Igush<i32> = Igush::new(5);
        array.extend_back(0..20);

        assert_eq!(array.get(0), Some(&0));
        assert_eq!(array.get(5), Some(&5));
        assert_eq!(array.get(19), Some(&19));
        assert_eq!(array.get(20), None);
    }

    #[test]
    fn get_mut() {
        let mut array: Igush<i32> = Igush::new(5);
        array.extend_back(0..20);

        assert_eq!(array.get_mut(0), Some(&mut 0));
        assert_eq!(array.get_mut(5), Some(&mut 5));
        assert_eq!(array.get_mut(19), Some(&mut 19));
        assert_eq!(array.get_mut(20), None);

        let x = array.get_mut(0).unwrap();
        *x = 22;

        assert_eq!(array.get_mut(0), Some(&mut 22));
    }

    #[test]
    fn insert() {
        let mut array: Igush<i32> = Igush::new(5);
        array.extend_back(0..4);

        for i in (102..112).rev() {
            array.insert(2, i);
        }

        assert_eq!(
            array,
            [0, 1, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 2, 3]
        );

        array.insert(0, 7);

        assert_eq!(array.get(0), Some(&7));

        array.insert(array.len(), 8);

        assert_eq!(array.get(array.len() - 1), Some(&8));
    }

    #[test]
    fn pop_back() {
        let mut array: Igush<i32> = Igush::new(5);

        assert_eq!(array.pop_back(), None);

        array.extend_back(0..20);
        assert_eq!(array.len(), 20);

        assert_eq!(array.pop_back(), Some(19));
        assert_eq!(array.pop_back(), Some(18));
        assert_eq!(array.pop_back(), Some(17));
        assert_eq!(array.pop_back(), Some(16));
        assert_eq!(array.pop_back(), Some(15));
        assert_eq!(array.pop_back(), Some(14));
        assert_eq!(array.pop_back(), Some(13));

        assert_eq!(array, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
    }

    #[test]
    fn pop_front() {
        let mut array: Igush<i32> = Igush::new(5);

        assert_eq!(array.pop_front(), None);

        array.extend_back(0..20);
        assert_eq!(array.len(), 20);

        assert_eq!(array.pop_front(), Some(0));
        assert_eq!(array.pop_front(), Some(1));
        assert_eq!(array.pop_front(), Some(2));
        assert_eq!(array.pop_front(), Some(3));
        assert_eq!(array.pop_front(), Some(4));
        assert_eq!(array.pop_front(), Some(5));

        assert_eq!(array, [6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]);
    }

    #[test]
    fn remove() {
        let mut array: Igush<i32> = Igush::new(5);

        assert_eq!(array.remove(0), None);

        array.extend_back(0..20);
        assert_eq!(array.len(), 20);

        assert_eq!(array.remove(10), Some(10));

        assert_eq!(
            array,
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 11, 12, 13, 14, 15, 16, 17, 18, 19]
        );

        assert_eq!(array.remove(18), Some(19));
        assert_eq!(array.remove(0), Some(0));
        assert_eq!(array.remove(20), None);

        assert_eq!(
            array,
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 11, 12, 13, 14, 15, 16, 17, 18]
        );

        let mut vec = igush![0; 200];

        for x in 0..100 {
            let elem = vec.remove(x);
            assert!(elem.is_some());
        }

        assert_eq!(vec.len(), 100);

        for x in (0..100).rev() {
            let elem = vec.remove(x);
            assert!(elem.is_some());
        }

        assert!(vec.is_empty());

        let n = 100;
        let mut vec = igush![0; n];

        for x in (0..n - 1).rev() {
            vec.remove(x);
        }
    }

    #[test]
    fn equal() {
        let mut a: Igush<i32> = Igush::new(5);
        let mut b: Igush<i32> = Igush::with_capacity(32, 0);

        a.extend_back(0..20);
        b.extend_front((0..20).rev());

        assert_eq!(a, b);

        let s = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
        ];
        assert_eq!(a, s);
        assert_eq!(b, s);
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn out_of_bounds_insert() {
        let mut array: Igush<i32> = Igush::new(5);

        array.insert(1, 3);
    }

    #[test]
    fn into_vec() {
        let mut array: Igush<i32> = Igush::new(5);

        for i in 0..7 {
            array.push_back(i);
        }
        for i in (7..15).rev() {
            array.push_front(i);
        }

        let v: Vec<i32> = array.into();
        assert_eq!(v, vec![7, 8, 9, 10, 11, 12, 13, 14, 0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn sendable() {
        fn can_send<T: Send>(_: T) {}

        let array: Igush<i32> = Igush::new(5);
        can_send(array);
    }
}
