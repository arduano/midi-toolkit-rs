use crate::{events::*, sequence::event::Delta};

use super::{errors::MIDIParseError, readers::TrackReader};

pub struct TrackParser<T: TrackReader> {
    reader: T,
    pushback: i16,
    prev_command: u8,
    ended: bool,
}

pub struct ParserCheckpoint {
    pushback: i16,
    prev_command: u8,
    reader_pos: u64,
    ended: bool,
}

impl ParserCheckpoint {
    pub fn reader_pos(&self) -> u64 {
        self.reader_pos
    }
}

impl<T: TrackReader> TrackParser<T> {
    pub fn from_checkpoint(reader: T, checkpoint: ParserCheckpoint) -> Self {
        assert_eq!(
            checkpoint.reader_pos,
            reader.pos(),
            "Checkpoint reader pos does not match reader pos"
        );

        Self {
            reader,
            pushback: checkpoint.pushback,
            prev_command: checkpoint.prev_command,
            ended: checkpoint.ended,
        }
    }

    pub fn new(reader: T) -> Self {
        Self {
            reader,
            pushback: -1,
            prev_command: 0,
            ended: false,
        }
    }

    fn read(&mut self) -> Result<u8, MIDIParseError> {
        if self.pushback != -1 {
            let p: u8 = self.pushback as u8;
            self.pushback = -1;
            return Ok(p);
        }
        self.reader.read()
    }

    fn read_fast(&mut self) -> Result<u8, MIDIParseError> {
        self.reader.read()
    }

    fn read_var_length(&mut self) -> Result<u64, MIDIParseError> {
        let mut n: u64 = 0;
        loop {
            let byte = self.read()?;
            n = (n << 7) | (byte & 0x7F) as u64;
            if (byte & 0x80) == 0 {
                break;
            }
        }
        return Ok(n);
    }
}

impl<T: TrackReader> Iterator for TrackParser<T> {
    type Item = Result<Delta<u64, Event>, MIDIParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        macro_rules! check {
            ($val:expr) => {
                match $val {
                    Ok(v) => v,
                    Err(e) => {
                        self.ended = true;
                        return Some(Err(e));
                    }
                }
            };
        }

        macro_rules! assert_len {
            ($size:expr) => {
                if check!(self.read_fast()) != $size {
                    return err!(MIDIParseError::CorruptEvent);
                }
            };
        }

        macro_rules! ret {
            ($val:expr) => {
                Some(Ok($val))
            };
        }

        macro_rules! err {
            ($val:expr) => {
                Some(Err($val))
            };
        }

        if self.ended {
            return None;
        }

        let delta = check!(self.read_var_length());
        let mut command = check!(self.read());
        if command < 0x80 {
            self.pushback = command as i16;
            command = self.prev_command;
        }
        self.prev_command = command;
        let comm = command & 0xF0;
        match comm {
            0x80 => {
                let channel = command & 0x0F;
                let key = check!(self.read());
                let _vel = check!(self.read_fast());
                return ret!(Event::new_delta_note_off_event(delta, channel, key));
            }
            0x90 => {
                let channel = command & 0x0F;
                let key = check!(self.read());
                let vel = check!(self.read_fast());
                if vel == 0 {
                    return ret!(Event::new_delta_note_off_event(delta, channel, key));
                }
                return ret!(Event::new_delta_note_on_event(delta, channel, key, vel));
            }
            0xA0 => {
                let channel = command & 0x0F;
                let key = check!(self.read());
                let vel = check!(self.read_fast());
                return ret!(Event::new_delta_polyphonic_key_pressure_event(
                    delta, channel, key, vel
                ));
            }
            0xB0 => {
                let channel = command & 0x0F;
                let controller = check!(self.read());
                let value = check!(self.read_fast());
                return ret!(Event::new_delta_control_change_event(
                    delta, channel, controller, value
                ));
            }
            0xC0 => {
                let channel = command & 0x0F;
                let program = check!(self.read());
                return ret!(Event::new_delta_program_change_event(
                    delta, channel, program
                ));
            }
            0xD0 => {
                let channel = command & 0x0F;
                let pressure = check!(self.read());
                return ret!(Event::new_delta_channel_pressure_event(
                    delta, channel, pressure
                ));
            }
            0xE0 => {
                let channel = command & 0x0F;
                let var1 = check!(self.read());
                let var2 = check!(self.read_fast());
                return ret!(Event::new_delta_pitch_wheel_change_event(
                    delta,
                    channel,
                    (((var2 as i16) << 7) | var1 as i16) - 8192
                ));
            }
            _ => match command {
                0xF0 => {
                    let mut data = Vec::new();
                    loop {
                        let byte = check!(self.read());
                        if byte == 0xF7 {
                            break;
                        }
                        data.push(byte);
                    }
                    data.shrink_to_fit();
                    return ret!(Event::new_delta_system_exclusive_message_event(delta, data));
                }
                0xF2 => {
                    let var1 = check!(self.read());
                    let var2 = check!(self.read_fast());
                    return ret!(Event::new_delta_song_position_pointer_event(
                        delta,
                        ((var2 as u16) << 7) | var1 as u16
                    ));
                }
                0xF3 => {
                    let pos = check!(self.read());
                    return ret!(Event::new_delta_song_select_event(delta, pos));
                }
                0xF6 => {
                    return ret!(Event::new_delta_tune_request_event(delta));
                }
                0xF7 => {
                    return ret!(Event::new_delta_end_of_exclusive_event(delta));
                }
                0xF8 => {
                    return ret!(Event::new_delta_end_of_exclusive_event(delta));
                }
                0xFF => {
                    let command = check!(self.read());
                    match command {
                        0x00 => {
                            assert_len!(2);
                            return ret!(Event::new_delta_track_start_event(delta));
                        }
                        0x01..=0x0A | 0x7F => {
                            let size = check!(self.read_var_length());
                            let mut data = Vec::new();
                            for _ in 0..size {
                                data.push(check!(self.read_fast()));
                            }
                            data.shrink_to_fit();

                            return ret!(Event::new_delta_text_event(
                                delta,
                                TextEventKind::from_val(command),
                                data
                            ));
                        }
                        0x20 => {
                            assert_len!(1);
                            let prefix = check!(self.read_fast());
                            return ret!(Event::new_delta_channel_prefix_event(delta, prefix));
                        }
                        0x21 => {
                            assert_len!(1);
                            let port = check!(self.read_fast());
                            return ret!(Event::new_delta_midi_port_event(delta, port));
                        }
                        0x2F => {
                            assert_len!(0);
                            self.ended = false;
                            return None;
                        }
                        0x51 => {
                            assert_len!(3);
                            let mut tempo: u32 = 0;
                            for _ in 0..3 {
                                tempo = (tempo << 8) | check!(self.read_fast()) as u32;
                            }
                            return ret!(Event::new_delta_tempo_event(delta, tempo));
                        }
                        0x54 => {
                            assert_len!(5);
                            let hr = check!(self.read_fast());
                            let mn = check!(self.read_fast());
                            let se = check!(self.read_fast());
                            let fr = check!(self.read_fast());
                            let ff = check!(self.read_fast());
                            return ret!(Event::new_delta_smpte_offset_event(
                                delta, hr, mn, se, fr, ff
                            ));
                        }
                        0x58 => {
                            assert_len!(4);
                            let nn = check!(self.read_fast());
                            let dd = check!(self.read_fast());
                            let cc = check!(self.read_fast());
                            let bb = check!(self.read_fast());
                            return ret!(Event::new_delta_time_signature_event(
                                delta, nn, dd, cc, bb
                            ));
                        }
                        0x59 => {
                            assert_len!(2);
                            let sf = check!(self.read_fast());
                            let mi = check!(self.read_fast());
                            return ret!(Event::new_delta_key_signature_event(delta, sf, mi));
                        }
                        _ => {
                            let size = check!(self.read_var_length());
                            let mut data = Vec::new();
                            for _ in 0..size {
                                data.push(check!(self.read_fast()));
                            }
                            data.shrink_to_fit();

                            return ret!(Event::new_delta_unknown_meta_event(delta, command, data));
                        }
                    }
                }
                _ => return ret!(Event::new_delta_undefined_event(delta, command)),
            },
        }
    }
}
