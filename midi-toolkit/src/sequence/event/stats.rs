use std::{ops::Deref, sync::Arc, time::Duration};

use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

use crate::{
    events::{Event, MIDIDelta, MIDIEventEnum, TempoEvent},
    num::MIDINum,
    pipe,
    sequence::{event::merge_events_array, to_vec, to_vec_result, wrap_ok},
};

use super::Delta;

struct ElementCountDebug(&'static str, usize);

impl std::fmt::Debug for ElementCountDebug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}; {}]", self.0, self.1)
    }
}

/// A struct to hold the statistics of a sequence.
#[derive(Clone)]
pub struct ChannelStatistics<D: MIDINum> {
    note_on_count: u64,
    note_off_count: u64,
    total_event_count: u64,
    total_length_ticks: D,
    tempo_events: Arc<[Delta<D, TempoEvent>]>,
}

impl<T: MIDINum> ChannelStatistics<T> {
    /// The number of note on events
    pub fn note_on_count(&self) -> u64 {
        self.note_on_count
    }

    /// Alias for [`note_on_count`](#method.note_on_count)
    pub fn note_count(&self) -> u64 {
        self.note_on_count()
    }

    /// The number of note off events
    pub fn note_off_count(&self) -> u64 {
        self.note_off_count
    }

    /// The number of events that are not note on and note off.
    ///
    /// Alias for ([`total_event_count()`](#method.total_event_count) - [`note_on_count()`](#method.note_on_count) - [`note_off_count()`](#method.note_off_count))
    pub fn other_event_count(&self) -> u64 {
        self.total_event_count - self.note_on_count - self.note_off_count
    }

    /// The total number of events
    pub fn total_event_count(&self) -> u64 {
        self.total_event_count
    }

    /// The sum of all delta times in each event
    pub fn total_length_ticks(&self) -> T {
        self.total_length_ticks
    }

    /// Calculate the length in seconds based on the tick length and the tempo events,
    /// as well as the ppq
    pub fn calculate_total_duration(&self, ppq: u16) -> Duration {
        tempo_sequence_get_duration(&self.tempo_events, ppq, self.total_length_ticks)
    }
}

impl<T: MIDINum> std::fmt::Debug for ChannelStatistics<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("ChannelStatistics")
            .field("note_on_count", &self.note_on_count)
            .field("note_off_count", &self.note_off_count)
            .field("total_event_count", &self.total_event_count)
            .field("other_event_count", &self.other_event_count())
            .field("total_length_ticks", &self.total_length_ticks)
            .field(
                "tempo_events",
                &ElementCountDebug("TempoEvent", self.tempo_events.len()),
            )
            .finish()
    }
}

/// A struct to hold the statistics of a group of sequences.
pub struct ChannelGroupStatistics<T: MIDINum> {
    group: ChannelStatistics<T>,
    channels: Vec<ChannelStatistics<T>>,
}

impl<T: MIDINum> ChannelGroupStatistics<T> {
    /// The list of statistics for individual channels
    pub fn channels(&self) -> &[ChannelStatistics<T>] {
        &self.channels
    }
}

impl<T: MIDINum> Deref for ChannelGroupStatistics<T> {
    type Target = ChannelStatistics<T>;

    fn deref(&self) -> &Self::Target {
        &self.group
    }
}

impl<T: MIDINum> std::fmt::Debug for ChannelGroupStatistics<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("ChannelGroupStatistics")
            .field("group", &self.group)
            .field(
                "channels",
                &ElementCountDebug("ChannelStatistics", self.channels.len()),
            )
            .finish()
    }
}

pub fn tempo_sequence_get_duration<T: MIDINum>(
    tempos: &[Delta<T, TempoEvent>],
    ppq: u16,
    ticks: T,
) -> Duration {
    let mut ticks = ticks;
    let mut time = 0.0;
    let mut multiplier = (500000.0 / ppq as f64) / 1000000.0;
    for t in tempos {
        let offset = t.delta();
        if offset > ticks {
            break;
        }
        ticks -= offset;

        let offset: f64 = offset.midi_num_into();
        time += multiplier * offset;
        multiplier = (t.tempo as f64 / ppq as f64) / 1000000.0;
    }
    let ticks: f64 = ticks.midi_num_into();
    time += multiplier * ticks;
    Duration::from_secs_f64(time)
}

/// Parse the events in a single channel and return the statistics for this channel.
///
/// ❗ **NOTE:** Time in seconds may be inaccurate due to the channel not having the MIDI's tempo events!
/// Make sure the iterator contains all of the MIDI's tempo events to get the accurate length in seconds.
pub fn get_channel_statistics<D: MIDINum, E: MIDIEventEnum + MIDIDelta<D>, Err>(
    iter: impl Iterator<Item = Result<E, Err>>,
) -> Result<ChannelStatistics<D>, Err> {
    let mut note_on_count = 0;
    let mut note_off_count = 0;
    let mut total_event_count = 0;
    let mut total_length_ticks = D::zero();
    let mut ticks_since_last_tempo = D::zero();

    let mut tempo_events = Vec::new();

    for event in iter {
        let event = event?;
        total_event_count += 1;
        total_length_ticks += event.delta();
        ticks_since_last_tempo += event.delta();
        match event.as_event() {
            Event::NoteOn(_) => note_on_count += 1,
            Event::NoteOff(_) => note_off_count += 1,
            Event::Tempo(t) => {
                let ev = *t.clone();
                let tempo_delta = Delta::new(ticks_since_last_tempo, ev);
                tempo_events.push(tempo_delta);
                ticks_since_last_tempo = D::zero();
            }
            _ => (),
        }
    }

    Ok(ChannelStatistics {
        note_on_count,
        note_off_count,
        total_event_count,
        total_length_ticks,
        tempo_events: tempo_events.into(),
    })
}

/// Parse the events in an array of channels (multithreaded) and return the statistics for all of the channels,
/// as well as the combined stats.
///
/// **NOTE:** This uses `rayon` for the threadpool, if you want to use your own rayon threadpool instance then
/// install it before calling this function.
pub fn get_channels_array_statistics<
    D: MIDINum,
    E: MIDIEventEnum + MIDIDelta<D>,
    Err: Send,
    I: Iterator<Item = Result<E, Err>> + Sized + Send,
>(
    iters: Vec<I>,
) -> Result<ChannelGroupStatistics<D>, Err> {
    let pool = iters
        .into_par_iter()
        .map(|iter| get_channel_statistics(iter));
    let mut result = Vec::new();
    pool.collect_into_vec(&mut result);
    let mut channels = pipe!(result.into_iter()|>to_vec_result())?;

    let tempo_vecs: Vec<_> = channels.iter().map(|c| c.tempo_events.clone()).collect();
    let tempo_iterators = tempo_vecs
        .into_iter()
        .map(|tempos| pipe!(tempos.iter().cloned()|>wrap_ok()|>to_vec().into_iter()))
        .collect();

    let merge = pipe!(tempo_iterators|>merge_events_array()|>to_vec_result().unwrap());

    let tempo_events: Arc<[Delta<D, TempoEvent>]> = merge.into();

    for c in channels.iter_mut() {
        c.tempo_events = tempo_events.clone();
    }

    let mut max_tick_length = D::zero();
    for c in channels.iter() {
        if c.total_length_ticks > max_tick_length {
            max_tick_length = c.total_length_ticks;
        }
    }

    let group = ChannelStatistics {
        note_on_count: channels.iter().map(|c| c.note_on_count).sum(),
        note_off_count: channels.iter().map(|c| c.note_off_count).sum(),
        total_event_count: channels.iter().map(|c| c.total_event_count).sum(),
        total_length_ticks: max_tick_length,
        tempo_events,
    };

    Ok(ChannelGroupStatistics { group, channels })
}
