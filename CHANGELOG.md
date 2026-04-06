# Changelog

## 0.3.1

- Added `filter_map_events` to the event sequence adapters and extension traits, with delta carry semantics for dropped events.

## 0.3.0

- Added `InterleavedTempFile` as a shared temporary backing store with independent virtual streams and readers.
- Added `StagedMIDIWriter` for writing track data into a `.parts.*` sidecar before assembling the final MIDI file.
- Refactored `MIDIWriter` and `StagedMIDIWriter` onto a shared internal track-writing abstraction to reduce duplicate logic.
- Added concurrency, ordering, and stress coverage for the new temporary file and staged writer paths.
- Tightened writer validation so negative track ids now return a typed `InvalidTrackId` error.
- Expanded `MIDIWriteError` with explicit virtual-file and track-length failure modes.

## 0.2.0

- Added extension-trait based sequence APIs through `midi_toolkit::prelude::*`.
- Deprecated the `pipe!` macro in favor of method chaining.
- Added `MIDIWriter::new_from_stream` and kept `new_from_stram` as a deprecated compatibility shim.
- Added fallible writer entry points such as `try_open_track`, `try_open_next_track`, and `try_end`.
- Added typed writer state errors instead of relying only on panics.
- Fixed SMF SysEx serialization to emit `F0 + varlen length + payload`.
- Switched event and writer output paths to `write_all` semantics.
- Made invalid track access return `None` instead of panicking.
- Fixed the Windows-only `player-example` so `cargo clippy --all-targets --all-features` stays green on non-Windows hosts.
