// TODO: use a custom Deque based on slices instead of Vec
// that way we can use a single Vec instead of one per row
use std::collections::VecDeque;

/// Array with Constant Time Access and Fast Insertion and Deletion
/// compromise in performance between array and list
#[derive(Debug)]
pub struct Igush<T> {
    /// backing storage of the structure
    backing: Vec<VecDeque<T>>,
    /// capacity of each internal DEQ
    /// should be set to approximately `sqrt(N)` where `N` is the total capacity
    row_capacity: usize,
    /// current total data capacity
    capacity: usize,
    /// number of used DEQs
    rows: usize,
    /// current number of elements in array
    length: usize,
}

impl<T> Igush<T> {
    /// create a new array with the given row width and total capacity
    pub fn with_row_capacity(row_capacity: usize, total_capacity: usize) -> Igush<T> {
        // number of rows
        let rows = total_capacity / row_capacity
            + if total_capacity % row_capacity > 0 {
                1
            } else {
                0
            };

        let backing: Vec<VecDeque<T>> = (0..rows)
            .map(|_| VecDeque::with_capacity(row_capacity))
            .collect();
        let actual_total_capacity = backing.capacity() * row_capacity;

        Igush {
            backing,
            row_capacity,
            capacity: actual_total_capacity,
            rows: 0,
            length: 0,
        }
    }
    /// create a new array with a default row width of 10
    pub fn new() -> Igush<T> {
        Self::with_row_capacity(10, 0)
    }

    /// add a new DEQ if an insertion will overflow the current end DEQ
    fn grow_if_necessary(&mut self) {
        if self.rows == 0 {
            let end = VecDeque::with_capacity(self.row_capacity);
            self.backing.push(end);
            self.rows = self.backing.len();

            self.capacity = self.backing.capacity() * self.row_capacity;
        }

        // exit early if there are available empty DEQs
        if self.rows < self.backing.len() {
            return;
        }

        // check if end DEQ is full
        if self.backing[self.rows - 1].len() == self.row_capacity {
            let end = VecDeque::with_capacity(self.row_capacity);
            self.backing.push(end);
            self.rows = self.backing.len();

            self.capacity = self.backing.capacity() * self.row_capacity;
        }
    }

    /// number of elements stored in the array
    pub fn len(&self) -> usize {
        self.length
    }

    /// returns true if the array is empty
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// returns the number of elements the array can hold without reallocating
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// insert an element at the end of the array
    pub fn push_back(&mut self, element: T) {
        self.grow_if_necessary();

        self.backing[self.rows - 1].push_back(element);

        self.length += 1;
    }

    /// insert an element at the beginning of the array
    pub fn push_front(&mut self, element: T) {
        self.grow_if_necessary();

        for row in (1..self.rows).rev() {
            let popped = self.backing[row - 1].pop_back().unwrap();
            self.backing[row].push_front(popped);
        }
        self.backing[0].push_front(element);

        self.length += 1;
    }

    /// insert an element at an arbitrary position
    pub fn insert(&mut self, index: usize, element: T) {
        assert!(index <= self.len(), "index out of bounds");
        self.grow_if_necessary();

        let target_row = index / self.row_capacity;
        let column = index % self.row_capacity;

        if self.backing[target_row].len() == self.row_capacity {
            for row in (target_row + 1)..self.rows {
                let back = self.backing[row - 1].pop_back().unwrap();
                self.backing[row].push_front(back);
            }
        }
        self.backing[target_row].insert(column, element);

        self.length += 1;
    }

    /// remove and return the element at the end of the array
    pub fn pop_back(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let element = self.backing[self.rows - 1].pop_back();

        if element.is_some() {
            self.length -= 1;
        }

        element
    }

    /// remove and return the element at the beginning of the array
    pub fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let element = self.backing[0].pop_front();

        for row in 1..self.rows {
            let front = self.backing[row].pop_front().unwrap();
            self.backing[row - 1].push_back(front);
        }

        if element.is_some() {
            self.length -= 1;
        }

        element
    }

    /// remove and return an element in the array by index
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len() {
            return None;
        }

        let target_row = index / self.row_capacity;
        let column = index % self.row_capacity;

        let element = self.backing[target_row].remove(column);

        for row in (target_row + 1)..self.rows {
            let front = self.backing[row].pop_front().unwrap();
            self.backing[row - 1].push_back(front);
        }

        if element.is_some() {
            self.length -= 1;
        }

        element
    }

    /// retrieves an element in the array mutably by index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        let target_row = index / self.row_capacity;
        let column = index % self.row_capacity;

        self.backing
            .get_mut(target_row)
            .and_then(|x: &mut VecDeque<T>| x.get_mut(column))
    }

    /// retrieves an element in the array by index
    pub fn get(&mut self, index: usize) -> Option<&T> {
        let target_row = index / self.row_capacity;
        let column = index % self.row_capacity;

        self.backing
            .get(target_row)
            .and_then(|x: &VecDeque<T>| x.get(column))
    }
}

impl<T: PartialEq> PartialEq for Igush<T> {
    fn eq(&self, other: &Self) -> bool {
        self.backing.eq(&other.backing)
    }
}
impl<T: Eq> Eq for Igush<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let created: Igush<i32> = Igush::with_row_capacity(32, 0);

        assert_eq!(created.capacity(), 0);
        assert_eq!(created.len(), 0);

        let created: Igush<i32> = Igush::with_row_capacity(5, 5);

        assert_eq!(created.capacity(), 5);
        assert_eq!(created.len(), 0);

        let created: Igush<i32> = Igush::with_row_capacity(10, 11);

        assert_eq!(created.capacity(), 20);
        assert_eq!(created.len(), 0);
    }

    #[test]
    fn push_back() {
        let mut array: Igush<i32> = Igush::with_row_capacity(5, 0);

        for _ in 0..20 {
            array.push_back(1);
        }

        assert_eq!(array.len(), 20);
    }

    #[test]
    fn push_front() {
        let mut array: Igush<i32> = Igush::with_row_capacity(5, 0);

        for i in 0..20 {
            array.push_front(i);
        }

        assert_eq!(array.len(), 20);
    }

    #[test]
    fn get() {
        let mut array: Igush<i32> = Igush::with_row_capacity(5, 0);

        assert_eq!(array.get(0), None);

        array.push_back(1);

        assert_eq!(array.get(0), Some(&1));
    }

    #[test]
    fn get_mut() {
        let mut array: Igush<i32> = Igush::with_row_capacity(5, 0);

        assert_eq!(array.get_mut(0), None);

        array.push_back(1);

        assert_eq!(array.get_mut(0), Some(&mut 1));

        let x = array.get_mut(0).unwrap();
        *x = 5;

        assert_eq!(array.get_mut(0), Some(&mut 5));
    }

    #[test]
    fn insert() {
        let mut array: Igush<i32> = Igush::with_row_capacity(5, 0);

        array.push_back(0);
        array.push_back(1);
        array.push_back(2);
        array.push_back(3);

        for i in (102..122).rev() {
            array.insert(2, i);
        }

        assert_eq!(array.len(), 24);
        assert_eq!(array.get(0), Some(&0));
        assert_eq!(array.get(1), Some(&1));

        for i in 2..22 {
            let num = 100 + (i as i32);
            assert_eq!(array.get(i), Some(&num));
        }

        assert_eq!(array.get(22), Some(&2));
        assert_eq!(array.get(23), Some(&3));

        array.insert(0, 7);

        assert_eq!(array.get(0), Some(&7));

        array.insert(array.len(), 8);

        assert_eq!(array.get(array.len() - 1), Some(&8));
    }

    #[test]
    fn pop_back() {
        let mut array: Igush<i32> = Igush::with_row_capacity(5, 0);

        assert_eq!(array.pop_back(), None);

        for i in 0..20 {
            array.push_back(i);
        }

        assert_eq!(array.len(), 20);

        assert_eq!(array.pop_back(), Some(19));

        assert_eq!(array.len(), 19);
    }

    #[test]
    fn pop_front() {
        let mut array: Igush<i32> = Igush::with_row_capacity(5, 0);

        assert_eq!(array.pop_front(), None);

        for i in 0..20 {
            array.push_back(i);
        }

        assert_eq!(array.len(), 20);

        assert_eq!(array.pop_front(), Some(0));

        assert_eq!(array.len(), 19);
    }

    #[test]
    fn remove() {
        let mut array: Igush<i32> = Igush::with_row_capacity(5, 0);

        assert_eq!(array.remove(0), None);

        for i in 0..20 {
            array.push_back(i);
        }

        assert_eq!(array.len(), 20);

        assert_eq!(array.remove(10), Some(10));

        assert_eq!(array.len(), 19);

        assert_eq!(array.remove(18), Some(19));

        assert_eq!(array.remove(0), Some(0));

        assert_eq!(array.remove(20), None);
    }

    #[test]
    fn equal() {
        let mut a: Igush<i32> = Igush::with_row_capacity(5, 0);
        let mut b: Igush<i32> = Igush::with_row_capacity(5, 0);

        for i in 0..20 {
            a.push_back(i);
        }
        for i in (0..20).rev() {
            b.push_front(i);
        }

        assert_eq!(a, b);
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn out_of_bounds_insert() {
        let mut array: Igush<i32> = Igush::with_row_capacity(5, 0);

        array.insert(1, 3);
    }
}
