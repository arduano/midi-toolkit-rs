use std::{
    fmt::Display,
    ops::{AddAssign, DivAssign, MulAssign, SubAssign},
};

use num_traits::Num;
use std::fmt::Debug;

pub trait MIDINumInto<T: MIDINum> {
    /// Casts the midi time type to another supported type.
    ///
    /// By default, supports: i32, i64, u32, u64, f32, f64
    /// ## Example
    /// ```
    ///use midi_toolkit::num::{MIDINumInto};
    ///
    ///let dt_i32: i32 = 10;
    ///let dt_u64: u64 = 10;
    ///
    ///let dt_f32: f32 = dt_i32.midi_num_into();
    ///let dt_f64: f64 = dt_i32.midi_num_into();
    ///let dt_u32: u32 = dt_u64.midi_num_into();
    ///let dt_i64: i64 = dt_u64.midi_num_into();
    ///
    ///assert_eq!(dt_f32, 10f32);
    ///assert_eq!(dt_f64, 10f64);
    ///assert_eq!(dt_u32, 10u32);
    ///assert_eq!(dt_i64, 10i64);
    /// ```
    fn midi_num_into(&self) -> T;
}

pub trait MIDINumFrom<T: MIDINum> {
    /// Casts the midi time type to another supported type.
    ///
    /// By default, supports: i32, i64, u32, u64, f32, f64
    /// ## Example
    /// ```
    ///use midi_toolkit::num::{MIDINumInto};
    ///
    ///let dt_i32: i32 = 10;
    ///let dt_u64: u64 = 10;
    ///
    ///let dt_f32: f32 = dt_i32.midi_num_into();
    ///let dt_f64: f64 = dt_i32.midi_num_into();
    ///let dt_u32: u32 = dt_u64.midi_num_into();
    ///let dt_i64: i64 = dt_u64.midi_num_into();
    ///
    ///assert_eq!(dt_f32, 10f32);
    ///assert_eq!(dt_f64, 10f64);
    ///assert_eq!(dt_u32, 10u32);
    ///assert_eq!(dt_i64, 10i64);
    /// ```
    fn midi_num_from(val: T) -> Self;
}

pub trait MIDINum:
    Num
    + PartialOrd
    + PartialEq
    + AddAssign
    + SubAssign
    + DivAssign
    + MulAssign
    + Copy
    + Sized
    + Debug
    + Display
    + Send
    + Sync
    + MIDINumFrom<i32>
    + MIDINumFrom<f32>
    + MIDINumFrom<f64>
    + MIDINumFrom<i64>
    + MIDINumFrom<u32>
    + MIDINumFrom<u64>
    + MIDINumInto<i32>
    + MIDINumInto<f32>
    + MIDINumInto<f64>
    + MIDINumInto<i64>
    + MIDINumInto<u32>
    + MIDINumInto<u64>
{
}

macro_rules! impl_delta_from_to {
    ($from:ident, $to:ident) => {
        impl MIDINumInto<$to> for $from {
            fn midi_num_into(&self) -> $to {
                *self as $to
            }
        }

        impl MIDINumFrom<$to> for $from {
            fn midi_num_from(val: $to) -> Self {
                val as $from
            }
        }
    };
}

macro_rules! impl_delta_from {
    ($from:ident) => {
        impl_delta_from_to!($from, i32);
        impl_delta_from_to!($from, u32);
        impl_delta_from_to!($from, i64);
        impl_delta_from_to!($from, u64);
        impl_delta_from_to!($from, f32);
        impl_delta_from_to!($from, f64);
    };
}

impl_delta_from!(i32);
impl_delta_from!(u32);
impl_delta_from!(i64);
impl_delta_from!(u64);
impl_delta_from!(f32);
impl_delta_from!(f64);

impl MIDINum for i32 {}
impl MIDINum for u32 {}
impl MIDINum for i64 {}
impl MIDINum for u64 {}
impl MIDINum for f32 {}
impl MIDINum for f64 {}

#[cfg(test)]
mod tests {
    use crate::num::{MIDINumFrom, MIDINumInto};
    #[test]
    fn casts_delta_into() {
        let dt_i32: i32 = 10;
        let dt_u64: u64 = 10;

        let dt_f32: f32 = dt_i32.midi_num_into();
        let dt_f64: f64 = dt_i32.midi_num_into();
        let dt_u32: u32 = dt_u64.midi_num_into();
        let dt_i64: i64 = dt_u64.midi_num_into();

        assert_eq!(dt_f32, 10f32);
        assert_eq!(dt_f64, 10f64);
        assert_eq!(dt_u32, 10u32);
        assert_eq!(dt_i64, 10i64);
    }

    #[test]
    fn casts_delta_from() {
        let dt_i32: i32 = 10;
        let dt_u64: u64 = 10;

        let dt_f32 = f32::midi_num_from(dt_i32);
        let dt_f64 = f64::midi_num_from(dt_i32);
        let dt_u32 = u32::midi_num_from(dt_u64);
        let dt_i64 = i64::midi_num_from(dt_u64);

        assert_eq!(dt_f32, 10f32);
        assert_eq!(dt_f64, 10f64);
        assert_eq!(dt_u32, 10u32);
        assert_eq!(dt_i64, 10i64);
    }
}
