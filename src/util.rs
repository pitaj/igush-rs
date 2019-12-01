//! utility functions

/// Simultaneous truncated integer division and modulus.
/// Returns `(quotient, remainder)`.
pub fn div_rem(dividend: usize, divisor: usize) -> (usize, usize) {
    (dividend / divisor, dividend % divisor)
}

/// Wrap around at end of row
pub fn wrap_add(length: usize, split: usize, other: isize) -> usize {
    let other_abs = other.abs() as usize;
    if other < 0 {
        if other_abs > split {
            (split + length) - other_abs
        } else {
            split - other_abs
        }
    } else {
        (split + other_abs) % length
    }
}

/// Will re-order the slice to make this row contiguous
pub fn make_contiguous<T>(slice: &mut [T], split: usize) {
    debug_assert_ne!(split, 0, "already contiguous");

    // head smaller
    //             S
    //      [7 8 9 0 1 2 3 4 5 6]
    //       rotate_left(3)
    //      [0 1 2 3 4 5 6 7 8 9]
    // 
    // tail smaller
    //                   S
    //      [4 5 6 7 8 9 0 1 2 3]
    //       rotate_right(4)
    //      [0 1 2 3 4 5 6 7 8 9]
    // 
    
    // first, figure out whether the tail or head is smaller
    let head_len = split;
    let tail_len = slice.len() - split;

    if tail_len <= head_len {
        slice.rotate_right(tail_len);
    } else {
        slice.rotate_left(head_len);
    }
}
