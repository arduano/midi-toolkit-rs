# Midi Toolkit for Rust

A library for ultra high performance MIDI operations, designed for black MIDI.

I wrote this library as my first ever Rust project, and it's evolved since then into something fairly advanced, but it still has some rough spots.

The documentation is also very patchy. I can update the documentation if someone actually needs this library and I get time to work on it.

This crate still requires nightly Rust for coroutine support.

See the `midi-toolkit/examples` folder for usage examples. The `player` example is intentionally gated behind the `player-example` feature because it depends on an unpublished external crate.
The player example is Windows-only at runtime and uses the external `kdmapi` backend. On other platforms it compiles to a stub that prints a short availability note, which keeps `cargo check --all-targets --all-features` working cleanly.
