use std::fmt::Debug;

use crate::{
    events::{BatchTempo, CastEventDelta, MIDIDelta, MIDIEvent, MIDIEventEnum},
    notes::{MIDINote, Note},
    num::MIDINum,
};

use super::{
    common::{to_vec_result, unwrap_items, wrap_ok, TimeCaster},
    conversion::{events_to_notes, notes_to_events},
    event::{
        cancel_tempo_events, convert_events_into_batches, filter_events, filter_map_events,
        filter_non_note_events, filter_note_events, get_channel_statistics,
        get_channels_array_statistics, into_track_events, merge_events_array, scale_event_ppq,
        scale_event_time, ChannelGroupStatistics, ChannelStatistics, Delta, EventBatch, Track,
    },
    note::merge_notes_iterator,
};

pub trait IntoOkExt: Iterator + Sized {
    fn into_ok(self) -> impl Iterator<Item = Result<Self::Item, ()>> {
        wrap_ok(self)
    }
}

impl<I: Iterator> IntoOkExt for I {}

pub trait ResultIterExt<T, Err>: Iterator<Item = Result<T, Err>> + Sized {
    fn unwrap_items(self) -> impl Iterator<Item = T>
    where
        T: Debug,
        Err: Debug,
    {
        unwrap_items(self)
    }

    fn collect_vec_result(self) -> Result<Vec<T>, Err> {
        to_vec_result(self)
    }
}

impl<T, Err, I> ResultIterExt<T, Err> for I where I: Iterator<Item = Result<T, Err>> + Sized {}

pub trait EventSequenceExt<D: MIDINum, E, Err>:
    Iterator<Item = Result<Delta<D, E>, Err>> + Sized
{
    fn cast_event_delta<ND: MIDINum>(
        self,
    ) -> impl Iterator<Item = Result<<Delta<D, E> as CastEventDelta<ND>>::Output, Err>>
    where
        Delta<D, E>: CastEventDelta<ND>,
    {
        TimeCaster::<ND>::cast_event_delta(self)
    }

    fn scale_event_time(self, multiplier: D) -> impl Iterator<Item = Result<Delta<D, E>, Err>>
    where
        Delta<D, E>: MIDIDelta<D>,
    {
        scale_event_time(self, multiplier)
    }

    fn scale_event_ppq(self, from: D, to: D) -> impl Iterator<Item = Result<Delta<D, E>, Err>>
    where
        Delta<D, E>: MIDIEvent + MIDIDelta<D>,
    {
        scale_event_ppq(self, from, to)
    }

    fn filter_events(
        self,
        predicate: impl Fn(&Delta<D, E>) -> bool,
    ) -> impl Iterator<Item = Result<Delta<D, E>, Err>>
    where
        Delta<D, E>: MIDIEventEnum + MIDIDelta<D>,
    {
        filter_events(self, predicate)
    }

    fn filter_map_events<NE>(
        self,
        mapper: impl FnMut(E) -> Option<NE>,
    ) -> impl Iterator<Item = Result<Delta<D, NE>, Err>> {
        filter_map_events(self, mapper)
    }

    fn filter_note_events(self) -> impl Iterator<Item = Result<Delta<D, E>, Err>>
    where
        Delta<D, E>: MIDIEventEnum + MIDIDelta<D>,
    {
        filter_note_events(self)
    }

    fn filter_non_note_events(self) -> impl Iterator<Item = Result<Delta<D, E>, Err>>
    where
        Delta<D, E>: MIDIEventEnum + MIDIDelta<D>,
    {
        filter_non_note_events(self)
    }

    fn cancel_tempo_events(self, new_tempo: u32) -> impl Iterator<Item = Result<Delta<D, E>, Err>>
    where
        Delta<D, E>: BatchTempo + MIDIDelta<D>,
    {
        cancel_tempo_events(self, new_tempo)
    }

    fn event_batches(self) -> impl Iterator<Item = Result<Delta<D, EventBatch<E>>, Err>>
    where
        Delta<D, E>: MIDIDelta<D>,
    {
        convert_events_into_batches(self)
    }

    fn into_track_events(
        self,
        track: u32,
    ) -> impl Iterator<Item = Result<Delta<D, Track<E>>, Err>> {
        into_track_events(self, track)
    }

    fn events_to_notes(self) -> impl Iterator<Item = Result<Note<D>, Err>>
    where
        E: MIDIEventEnum,
    {
        events_to_notes(self)
    }

    fn channel_statistics(self) -> Result<ChannelStatistics<D>, Err>
    where
        Delta<D, E>: MIDIEventEnum + MIDIDelta<D>,
    {
        get_channel_statistics(self)
    }
}

impl<D: MIDINum, E, Err, I> EventSequenceExt<D, E, Err> for I where
    I: Iterator<Item = Result<Delta<D, E>, Err>> + Sized
{
}

pub trait EventSequenceCollectionExt<D: MIDINum, T, Err, I>: Iterator<Item = I> + Sized
where
    T: MIDIDelta<D>,
    I: Iterator<Item = Result<T, Err>> + Sized,
{
    fn merge_all(self) -> impl Iterator<Item = Result<T, Err>> {
        merge_events_array(self.collect())
    }

    fn channel_statistics(self) -> Result<ChannelGroupStatistics<D>, Err>
    where
        T: MIDIEventEnum + MIDIDelta<D>,
        Err: Send,
        I: Send,
    {
        get_channels_array_statistics(self.collect())
    }
}

impl<D: MIDINum, T, Err, I, II> EventSequenceCollectionExt<D, T, Err, I> for II
where
    T: MIDIDelta<D>,
    I: Iterator<Item = Result<T, Err>> + Sized,
    II: Iterator<Item = I> + Sized,
{
}

pub trait NoteSequenceExt<D: MIDINum, N, Err>: Iterator<Item = Result<N, Err>> + Sized
where
    N: MIDINote<D>,
{
    fn notes_to_events(self) -> impl Iterator<Item = Result<Delta<D, crate::events::Event>, Err>> {
        notes_to_events(self)
    }
}

impl<D: MIDINum, N, Err, I> NoteSequenceExt<D, N, Err> for I
where
    N: MIDINote<D>,
    I: Iterator<Item = Result<N, Err>> + Sized,
{
}

pub trait NoteSequenceCollectionExt<D: MIDINum, N, Err, I>: Iterator<Item = I> + Sized
where
    N: MIDINote<D>,
    I: Iterator<Item = Result<N, Err>> + Sized,
{
    fn merge_all(self) -> impl Iterator<Item = Result<N, Err>> {
        merge_notes_iterator(self)
    }
}

impl<D: MIDINum, N, Err, I, II> NoteSequenceCollectionExt<D, N, Err, I> for II
where
    N: MIDINote<D>,
    I: Iterator<Item = Result<N, Err>> + Sized,
    II: Iterator<Item = I> + Sized,
{
}
