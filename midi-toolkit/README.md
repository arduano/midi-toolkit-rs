# Midi Toolkit for Rust

A library for ultra high performance MIDI operations, designed for black MIDI.

I wrote this library as my first ever Rust project, and it grew into a fairly capable MIDI utility crate. The current API direction is to use extension traits from `midi_toolkit::prelude::*` instead of the old `pipe!` macro.

This crate still requires nightly Rust because its iterator pipeline internals are coroutine-based.

```rust
use midi_toolkit::{io::MIDIFile, prelude::*};

let file = MIDIFile::open("example.mid", None)?;
let merged = file.iter_all_tracks().merge_all().unwrap_items();
```

The `pipe!` macro remains available for compatibility, but it is deprecated.

See the `midi-toolkit/examples` folder for some examples of how to use the library.
The `player` example is Windows-only at runtime and depends on the external `kdmapi` backend; outside that environment it compiles to a stub so the crate still checks cleanly under `--all-features`.
