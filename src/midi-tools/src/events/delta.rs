use num_traits::Num;

pub trait DeltaNumInto<T: DeltaNum> {
    fn delta_into(&self) -> T;
}

pub trait DeltaNum: Num + Copy + Sized {}

macro_rules! impl_delta_from_to {
    ($from:ident, $to:ident) => {
        impl DeltaNumInto<$to> for $from {
            fn delta_into(&self) -> $to {
                *self as $to
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

impl DeltaNum for i32 {}
impl DeltaNum for u32 {}
impl DeltaNum for i64 {}
impl DeltaNum for u64 {}
impl DeltaNum for f32 {}
impl DeltaNum for f64 {}
