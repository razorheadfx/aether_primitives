use std::ops::{Add, Mul};

struct Fir<T>
where
    T: Mul + Add + Default,
{
    taps: Vec<T>,
    tmp: Vec<T>,
}

impl<T> Fir<T>
where
    T: Mul + Add + Default,
{
    fn new(taps: Vec<T>, input_len: usize) -> Fir<T> {
        let filter_len = taps.len() + input_len;
        Fir {
            taps,
            tmp: Vec::with_capacity(filter_len),
        }
    }
}
