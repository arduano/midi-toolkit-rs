// use gen_iter::GenIter;

// use crate::{notes::MIDINote, num::MIDINum, unwrap, yield_error};

// /// Merge an array of note iterators together into one iterator.
// pub fn merge_notes_iterator<
//     T: MIDINum,
//     N: MIDINote<T>,
//     Err,
//     I: Iterator<Item = Result<N, Err>> + Sized,
//     II: Iterator<Item = I> + Sized,
// >(
//     mut iter: II,
// ) -> impl Iterator<Item = Result<N, Err>> {
//     struct SeqTime<T: MIDINum, N: MIDINote<T>, Err, I: Iterator<Item = Result<N, Err>> + Sized> {
//         iter: I,
//         time: T,
//         next: Option<N>,
//     }

//     let vector = Vec::<i32>::new();

//     let moved1 = vector;
//     let moved2 = vector;

//     GenIter(move || {
//         let mut get_next_seq = move || {
//             while let Some(mut seq) = iter.next() {
//                 let first = seq.next();
//                 match first {
//                     None => continue,
//                     Some(e) => match e {
//                         Err(e) => return Err(e),
//                         Ok(e) => {
//                             let s = SeqTime {
//                                 time: e.start(),
//                                 next: Some(e),
//                                 iter: seq,
//                             };

//                             return Ok(Some(s));
//                         }
//                     },
//                 }
//             }
//             Ok(None)
//         };

//         let first_seq = match get_next_seq() {
//             Err(e) => {
//                 yield Err(e);
//                 None
//             }
//             Ok(e) => e,
//         };

//         if let Some(first_seq) = first_seq {
//             let mut seqences = Vec::new();
//             seqences.push(first_seq);

//             let mut next_seq = get_next_seq();

//             while seqences.len() > 0 {
//                 let len = seqences.len();
//                 let mut smallest_index = 0;
//                 let mut smallest_time = seqences[0].time;
//                 for i in 0..len {
//                     let next = &seqences[i];
//                     if next.time < smallest_time {
//                         smallest_time = next.time;
//                         smallest_index = i;
//                     }
//                 }
//                 loop {
//                     let (note, next) = {
//                         let smallest = &mut seqences[smallest_index];

//                         let note = smallest.next.take().unwrap();

//                         (note, smallest.iter.next())
//                     };
//                     yield Ok(note);
//                     match next {
//                         None => {
//                             seqences.remove(smallest_index);
//                             break;
//                         }
//                         Some(next) => {
//                             let next = unwrap!(next);
//                             let mut smallest = &mut seqences[smallest_index];
//                             smallest.time = next.start();
//                             smallest.next = Some(next);
//                         }
//                     }
//                     if seqences[smallest_index].time != smallest_time {
//                         break;
//                     }
//                 }
//             }
//         }
//     })
// }
