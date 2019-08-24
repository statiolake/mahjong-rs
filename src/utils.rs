use std::ops::{Add, Div, Mul, Sub};

pub fn ceil_at<T>(x: T, at: T) -> T
where
    T: Add<Output = T> + Sub<Output = T> + Div<Output = T> + Mul<Output = T> + From<u8> + Copy,
{
    (x + (at - 1.into())) / at * at
}
