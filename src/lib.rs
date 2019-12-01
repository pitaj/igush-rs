mod util;

/// Array with Constant Time Access and Fast Insertion and Deletion
/// compromise in performance between pure array and linked list
#[derive(Debug, Clone)]
pub struct Igush<T> {
    /// backing storage of the structure
    backing: Vec<T>,
    /// where each DEQ is split betwen head and tail
    splits: Vec<usize>,
    /// width of each DEQ
    row_width: usize,
}

use std::iter::{FusedIterator, repeat};
use std::mem::swap;
use std::cmp::min;

impl<T> Igush<T> {
    /// Create a new array from a `Vec`
    pub fn from(other: Vec<T>, row_width: usize) -> Self {
        // always at least one row
        let rows = other.len() / row_width + 1;
        
        Igush {
            backing: other,
            splits: repeat(0).take(rows).collect(),
            row_width
        }
    }
    /// create a new array with the given row width and total capacity
    pub fn with_capacity(row_width: usize, capacity: usize) -> Self {
        // always at least one row
        let rows = capacity / row_width + 1;

        Igush {
            backing: Vec::with_capacity(capacity),
            splits: Vec::with_capacity(rows),
            row_width,
        }
    }
    /// create a new array without allocating
    pub fn new(row_width: usize) -> Self {
        Self::with_capacity(row_width, 0)
    }

    /// number of elements stored in the array
    pub fn len(&self) -> usize {
        self.backing.len()
    }

    /// returns true if the array is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// returns the number of elements the array can hold without reallocating
    pub fn capacity(&self) -> usize {
        self.backing.capacity()
    }

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
            let end = start + self.row_width;
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

    /// insert an element at the end of the array
    pub fn push_back(&mut self, element: T) {
        // last DEQ is always kept contiguous
        // so pushing is just pushing to the Vec
        self.backing.push(element);

        self.correct_splits();
    }

    /// extend from back with the contents of an iterator
    pub fn extend_back<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for elem in iter {
            self.push_back(elem);
        }
    }

    /// insert an element at the beginning of the array
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

    /// extend from front with the contents of an iterator
    pub fn extend_front<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for elem in iter {
            self.push_front(elem);
        }
    }

    /// insert an element at an arbitrary position
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

    /// remove and return the element at the end of the array
    pub fn pop_back(&mut self) -> Option<T> {
        // last DEQ is always kept contiguous
        // so popping is just popping from the Vec
        let element = self.backing.pop();
        self.correct_splits();

        element
    }

    /// remove and return the element at the beginning of the array
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

    /// remove and return an element in the array by index
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

    /// Make the backing stucture completely contiguous.
    pub fn make_contiguous(&mut self) {
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
    }

    /// retrieves an element in the array by index
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len() {
            return None;
        }

        let i = self.real_index(index);
        self.backing.get(i)
    }

    /// retrieves an element in the array mutably by index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len() {
            return None;
        }

        let i = self.real_index(index);
        self.backing.get_mut(i)
    }

    /// Returns a front-to-back iterator.
    pub fn iter(&self) -> impl Iterator<Item = &T> + DoubleEndedIterator + ExactSizeIterator + FusedIterator {
        (0..self.len()).into_iter().map(move |index| {
            self.get(index).unwrap()
        })
    }

    /// Returns a front-to-back iterator that returns mutable references.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> + DoubleEndedIterator + ExactSizeIterator + FusedIterator {
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
}

impl<T: PartialEq> PartialEq for Igush<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}
impl<T: PartialEq> PartialEq<&[T]> for Igush<T> {
    fn eq(&self, other: &&[T]) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}
impl<T: Eq> Eq for Igush<T> {}

impl<T> Into<Vec<T>> for Igush<T> {
    fn into(mut self) -> Vec<T> {
        self.make_contiguous();
        self.backing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Easier equality comparison.
    /// ```
    /// equal!(igush_instance, [1, 2, 3, 4, 5, 6]);
    /// ```
    macro_rules! equal {
        ($actual:expr, $expected:expr) => {
            assert_eq!($actual, &$expected as &[_]);
        }
    }

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
        equal!(
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
        equal!(
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

        equal!(
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

        equal!(array, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
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

        equal!(array, [6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]);
    }

    #[test]
    fn remove() {
        let mut array: Igush<i32> = Igush::new(5);

        assert_eq!(array.remove(0), None);

        array.extend_back(0..20);
        assert_eq!(array.len(), 20);

        assert_eq!(array.remove(10), Some(10));

        equal!(
            array,
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 11, 12, 13, 14, 15, 16, 17, 18, 19]
        );

        assert_eq!(array.remove(18), Some(19));
        assert_eq!(array.remove(0), Some(0));
        assert_eq!(array.remove(20), None);

        equal!(
            array,
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 11, 12, 13, 14, 15, 16, 17, 18]
        );
    }

    #[test]
    fn equal() {
        let mut a: Igush<i32> = Igush::new(5);
        let mut b: Igush<i32> = Igush::with_capacity(32, 0);

        a.extend_back(0..20);
        b.extend_front((0..20).rev());

        assert_eq!(a, b);

        let s = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19];
        equal!(a, s);
        equal!(b, s);
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
}
