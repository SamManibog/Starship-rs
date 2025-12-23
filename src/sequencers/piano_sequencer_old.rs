use std::{cmp::Ordering, marker::PhantomData};

use crate::sequencers::note::Note;

/// the id of a note in a pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PatternNoteId {
    voice_index: usize,
    index: usize
}

/// a handle to a note, valid for as long as the pattern containing it
/// is not mutated
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NoteHandle<'a> {
    ptr: *mut Note,
    phantom: PhantomData<&'a ()>,
}

/// a pattern of notes data is stored as an interval tree
pub struct PianoPattern {
    /// each note belongs to a voice single voice
    /// notes within voices never overlap at any time
    /// voices are used to id notes, so its desirable to minimize
    /// the total amount of voices so as to keep ids small as possible for sequencing
    ///
    /// we try to prioritize putting notes in lower-indexed voices as much as possible
    voices: Vec<Vec<*mut Note>>
}

impl PianoPattern {
    /// O(n * (k + log n)) where k is the number of voices needed
    pub fn new(notes: Vec<Box<Note>>) -> Self {
        // convert notes into pointer list
        let mut notes: Vec<*mut Note> = notes.into_iter().map(|n| Box::into_raw(n)).collect();

        // sort notes by end time
        // UNWRAP SAFETY: the Note struct ensures that end_time is a real number
        notes.sort_unstable_by(|x, y| unsafe {
            (**x).end_time().partial_cmp(&(**y).end_time()).unwrap()
        });

        // use greedy algorithm for activity selection problem to fill voices
        let mut voices = Vec::new();
        while notes.len() > 0 {
            let mut voice: Vec<*mut Note> = Vec::new();
            let mut last_end = -1.0;
            notes.retain(|n| {
                if unsafe { (**n).start_time() } > last_end {
                    voice.push(*n);
                    last_end = unsafe { (**n).end_time() };
                    false
                } else {
                    true
                }
            });
            voices.push(voice);
        }

        Self {
            voices
        }
    }

    pub fn add_note(&mut self, note: Box<Note>) {
        let start_time = note.start_time();
        let end_time = note.end_time();

        // attempt to add note directly into voices
        for (mut vindex, voice) in self.voices.iter_mut().enumerate() {
            if unsafe { !Self::is_voice_active_in_range(&voice, start_time, end_time) } {
                // unwrap safety: there is no active note within start and end time,
                // so an error value is expected as we will fail to find another note
                let index = unsafe { Self::voice_note_at_time(&voice, start_time) }.unwrap_err();
                voice.insert(index, Box::into_raw(note));
                
                // preserve invariant that lower-indexed voices have the most notes
                while vindex > 0 && self.voices[vindex - 1].len() < self.voices[vindex].len() {
                    self.voices.swap(vindex - 1, vindex);
                    vindex -= 1;
                }
                return;
            }
        }

        // if we cant add it, create a new voice
        self.voices.push(vec![Box::into_raw(note)]);
    }

    /// deletes the note with the given id if it exists
    /// returns true on successful deletion and false otherwise
    pub fn delete_note(&mut self, id: PatternNoteId) -> bool {
        if let Some(voice) = self.voices.get_mut(id.voice_index) {
            if voice.len() <= id.index {
                return false;
            }

            voice.remove(id.index);

            // preserve invariant that lower-indexed voices have the most notes
        	let mut vindex = id.voice_index + 1;
            while vindex < self.voices.len() && self.voices[vindex - 1].len() < self.voices[vindex].len() {
                self.voices.swap(vindex - 1, vindex);
                vindex += 1;
            }

            true
        } else {
            false
        }
    }

    /// gets the handle to the note from its id
    pub fn get_note_handle(&self, id: PatternNoteId) -> Option<NoteHandle> {
        let ptr = *self.voices.get(id.voice_index)?.get(id.index)?;

        Some(NoteHandle {
            ptr,
            phantom: PhantomData {},
        })
    }

    /// gets the notes at the given time by id
    pub fn active_at_time(&self, time: f64) -> Vec<PatternNoteId> {
        let mut notes = Vec::new();
        for (voice_index, voice) in self.voices.iter().enumerate() {
            if let Ok(index) = unsafe { Self::voice_note_at_time(&voice, time) } {
                notes.push(PatternNoteId { voice_index, index });
            }
        }
        notes
    }

    /// checks if the pattern is active at the given time
    pub fn is_active_at_time(&self, time: f64) -> bool {
        for voice in &self.voices {
            if let Ok(_) = unsafe { Self::voice_note_at_time(&voice, time) } {
                return true;
            }
        }
        false
    }

    /// finds the note at the given time in the voice if it exists
    /// you must ensure the notes in the voice are valid
    unsafe fn voice_note_at_time(voice: &Vec<*mut Note>, time: f64) -> Result<usize, usize> {
        voice.binary_search_by(|n| {
            if unsafe { (**n).end_time() } < time {
                Ordering::Less
            } else if unsafe { (**n).start_time() } > time {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        })
    }

    /// checks if a note is playing in a voice in the given timeframe
    /// you must ensure the notes in the voice are valid
    unsafe fn is_voice_active_in_range(voice: &Vec<*mut Note>, start: f64, end: f64) -> bool {
        let start_idx_raw = unsafe { Self::voice_note_at_time(voice, start) };
        let end_idx_raw = unsafe { Self::voice_note_at_time(voice, end) };

        if let (Err(start_idx), Err(end_idx)) = (start_idx_raw, end_idx_raw) {
            // if start_idx == end_idx, the range is empty and the voice is not active.
            // otherwise, at least one note is contained so the voice is active sometime between
            // start and end
            start_idx == end_idx
        } else {
            // if either start or end time is on a note, the voice is active
            true
        }
    }
}

/* proposed implementation of piano pattern voices using btrees rather than vectors
/// a B-Tree that stores notes over disjoint time intervals
/// supports lookup based on both start and end times
///
/// reasoning for use of B-Tree:
///  - lookups will be the most frequent operation especially during playback
///  	the cache locality of B-Trees should help with this
///  - modifications are not as important as they are performed on human-time,
///  	so the performance hit of storing elements in lists is not as harmful
struct Voice {
    root: *mut VoiceNode,
}

impl Voice {
    /// searches for the note at the given time, returning it and its unique identifier
    fn note_at_time(&self, time: f64) -> Option<(&Note, u32)> {
        let mut id: u32 = 0;

        let mut node = self.root;
        loop {
            match unsafe { (*node).index_search_by_time(time) } {
                IndexSearchResult::Empty => { return None; }

                IndexSearchResult::Child(index) => {
                    id += index as u32;
                    id *= VoiceNode::ORDER as u32;
                    node = unsafe { (*node).children[index] };
                }

                IndexSearchResult::Note(index) => {
                    return Some((
                        unsafe { &*(*node).notes[index] },
                        id + index as u32
                    ));
                }
            }
        }
    }
}

/// A node in a voice's b-tree
struct VoiceNode {
    /// notes ordered by time
    notes: [*mut Note; Self::ORDER - 1],

    /// children between notes ordered by time
    children: [*mut VoiceNode; Self::ORDER],

    /// the number of children
    size: usize,
}

impl VoiceNode {
    /// the maximum number of children the node may have
    const ORDER: usize = 8;

    fn new() -> Self {
        Self {
            notes: [std::ptr::null_mut(); 7],
            children: [std::ptr::null_mut(); 8],
            size: 0
        }
    }

    /// searches for the index of the note by exact start time
    fn index_search_by_start(&self, start_time: f64) -> IndexSearchResult {
        self.index_search_by(|n| n.start_time().partial_cmp(&start_time).unwrap())
    }

    /// searches for the index of the note by exact end time
    fn index_search_by_end(&self, end_time: f64) -> IndexSearchResult {
        self.index_search_by(|n| n.end_time().partial_cmp(&end_time).unwrap())
    }

    /// searches for the index of the note by contained time
    fn index_search_by_time(&self, time: f64) -> IndexSearchResult {
        self.index_search_by(|n| {
            if n.end_time() < time {
                Ordering::Less
            } else if n.start_time() > time {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        })
    }

    /// searches for the index of note by the given function
    /// f(n) should return n's relation to the target
    fn index_search_by(&self, f: impl Fn(&Note) -> Ordering) -> IndexSearchResult {
        if self.size == 0 {
            return IndexSearchResult::Empty;
        }

        // the minimum index allowed to search (inclusive)
        let mut left = 0;

        // the maximum index allowed to search (exclusive)
        let mut right = self.size;

        // the last ordering of the function call
        let mut ord = Ordering::Equal;

        while left < right {
            let mid = (left + right - 1) / 2;
            ord = unsafe { f(&*self.notes[mid]) };
            match ord {
                Ordering::Less => {
                    left = mid + 1;
                },

                Ordering::Greater => {
                    right = mid;
                },

                Ordering::Equal => {
                    right = left;
                },
            }
        }

        // left == right

        match ord {
            Ordering::Less => {
                IndexSearchResult::Child(left + 1)
            },

            Ordering::Greater => {
                IndexSearchResult::Child(left)
            },

            Ordering::Equal => {
                IndexSearchResult::Note(left)
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum IndexSearchResult {
    Child(usize),
    Note(usize),
    Empty
}
*/



// TODO:
// 	When adding or deleting notes from a pattern,
// 	we need to find a way to rearrange the notes to reduce the number of voices
