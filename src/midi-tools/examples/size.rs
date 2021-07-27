use std::mem::size_of;

use midi_tools::events::Event;

enum test {
    aa(Box<i32>),
    ab,
    ac,
    ad,
    ba,
    bb,
    bc,
    bd,
    ca,
    cb,
    cc,
    cd,
    da,
    db,
    dc,
    dd,
}

pub fn main() {
    dbg!(size_of::<Event<i32>>());
    dbg!(size_of::<Event<f64>>());
    dbg!(size_of::<Box<i32>>());
    dbg!(size_of::<test>());
}
