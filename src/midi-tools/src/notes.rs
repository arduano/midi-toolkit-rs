use crate::num::MIDINum;

pub trait MIDINote<T: MIDINum> {
    fn start(&self) -> T;
    fn start_mut(&mut self) -> &mut T;
    fn len(&self) -> T;
    fn len_mut(&mut self) -> &mut T;

    fn key(&self) -> u8;
    fn key_mut(&mut self) -> &mut u8;
    #[inline(always)]
    fn set_key(&mut self, key: u8) {
        *self.key_mut() = key;
    }

    fn channel(&self) -> u8;
    fn channel_mut(&mut self) -> &mut u8;
    #[inline(always)]
    fn set_channel(&mut self, channel: u8) {
        *self.channel_mut() = channel;
    }

    fn velocity(&self) -> u8;
    fn velocity_mut(&mut self) -> &mut u8;
    #[inline(always)]
    fn set_velocity(&mut self, velocity: u8) {
        *self.velocity_mut() = velocity;
    }

    #[inline(always)]
    fn end(&self) -> T {
        self.start() + self.len()
    }

    /// Sets the note start and note length, keeping the end the same
    #[inline(always)]
    fn move_start(&mut self, start: T) {
        let end = self.end();
        self.set_start(start);
        self.set_len(end - start);
    }

    /// Sets the note length based on a new end, keeping the start the same
    #[inline(always)]
    fn set_end(&mut self, end: T) {
        self.set_len(end - self.start());
    }

    /// Sets the note start, keeping the length the same
    #[inline(always)]
    fn set_start(&mut self, start: T) {
        *self.start_mut() = start;
    }

    /// Sets the note length, keeping the start the same
    #[inline(always)]
    fn set_len(&mut self, len: T) {
        *self.len_mut() = len;
    }
}

pub struct Note<T: MIDINum> {
    pub start: T,
    pub len: T,
    pub key: u8,
    pub channel: u8,
    pub velocity: u8,
}

impl<T: MIDINum> MIDINote<T> for Note<T> {
    #[inline(always)]
    fn start(&self) -> T {
        self.start
    }

    #[inline(always)]
    fn start_mut(&mut self) -> &mut T {
        &mut self.start
    }

    #[inline(always)]
    fn len(&self) -> T {
        self.len
    }

    #[inline(always)]
    fn len_mut(&mut self) -> &mut T {
        &mut self.len
    }

    #[inline(always)]
    fn key(&self) -> u8 {
        self.key
    }

    #[inline(always)]
    fn key_mut(&mut self) -> &mut u8 {
        &mut self.key
    }

    #[inline(always)]
    fn channel(&self) -> u8 {
        self.channel
    }

    #[inline(always)]
    fn channel_mut(&mut self) -> &mut u8 {
        &mut self.channel
    }

    #[inline(always)]
    fn velocity(&self) -> u8 {
        self.velocity
    }

    #[inline(always)]
    fn velocity_mut(&mut self) -> &mut u8 {
        &mut self.velocity
    }
}
