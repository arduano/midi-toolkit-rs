# Changelog

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
