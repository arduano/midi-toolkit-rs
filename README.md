# Midi Toolkit for Rust

A library for ultra high performance MIDI operations, designed for black MIDI.

I wrote this library as my first ever Rust project, and it grew into a fairly capable MIDI utility crate. This branch is a cleanup pass aimed at making it safer and less macro-heavy to use.

The crate still targets nightly Rust because its iterator pipeline internals are coroutine-based.

## API direction

The intended API style is now method chaining through `midi_toolkit::prelude::*`:

```rust
use midi_toolkit::{io::MIDIFile, prelude::*};

let file = MIDIFile::open("example.mid", None)?;
let stats = file.iter_all_tracks().channel_statistics()?;
```

The old `pipe!` macro is still present for compatibility, but it is deprecated.

Writer code should prefer the fallible `try_*` entry points:

```rust
use midi_toolkit::io::MIDIWriter;

let writer = MIDIWriter::new("out.mid", 480)?;
let mut track = writer.try_open_next_track()?;
```

See the `midi-toolkit/examples` folder for usage examples. The `player` example is intentionally gated behind the `player-example` feature because it depends on an unpublished external crate.
The player example is Windows-only at runtime and uses the external `kdmapi` backend. On other platforms it compiles to a stub that prints a short availability note, which keeps `cargo check --all-targets --all-features` working cleanly.
