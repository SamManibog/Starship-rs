use core::f64;
use std::{cmp::Ordering, ops::{Add, AddAssign, Neg, Sub, SubAssign}, vec};

use thiserror::Error;

use crate::{pitch::DetunedPitch, sequencers::curve::CurveShape};

/// BeatUnits
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BeatUnits(pub i32);

impl BeatUnits {
    pub const MIN: BeatUnits = BeatUnits(i32::MIN);
    pub const MAX: BeatUnits = BeatUnits(i32::MAX);

	/// The amount of units per beat.
    /// 20160, divisible by 5, 7, 9, and 64.
    pub const UNITS_PER_BEAT: i32 = 5 * 7 * 9 * 64;

    pub const fn into_beats(&self) -> f64 {
        self.0 as f64 * Self::UNITS_PER_BEAT as f64
    }

    pub const fn from_beats(beats: f64) -> Self {
        let max_magnitude = Self::UNITS_PER_BEAT as f64 * i32::MAX as f64;
        Self((beats * Self::UNITS_PER_BEAT as f64).clamp(-max_magnitude, max_magnitude) as i32)
    }
}

impl Neg for BeatUnits {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl Add for BeatUnits {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for BeatUnits {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl AddAssign for BeatUnits {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

impl SubAssign for BeatUnits {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0
    }
}

/// the transition between partials (or fade in/out)
#[derive(Debug, Clone)]
pub struct NoteTransition {
    /// the shape of the transition
    pub shape: CurveShape,

    /// the time that the transition starts (inclusive)
    pub start_time: BeatUnits,

    /// the time that the transition ends (exclusive)
    pub end_time: BeatUnits,

    // the initial pitch of the transition
    pub start_pitch: DetunedPitch,

    // the final pitch of the transition
    pub end_pitch: DetunedPitch,
}

impl NoteTransition {
    /// gets the change in cents from a4 at the given time in beats
    pub fn get_cent_delta_a4(&self, time: f64) -> f64 {
        self.shape.interpolate(
            time,
            self.start_time.into_beats(),
            self.end_time.into_beats(),
            self.start_pitch.cent_delta_a4() as f64,
            self.end_pitch.cent_delta_a4() as f64
        )
    }
}

/// an error occurring when attempting to delete a partial note
#[derive(Debug, Error)]
pub enum DeleteNotePartialError {
    #[error("The end index is less than the start index.")]
    InvalidRange,

    #[error("Attempted to delete a partial not stored in the note.")]
    PartialOutOfBounds,

    #[error("Attempted to delete all partials in the note.  There must be at least one partial.")]
    FullClearError,

    #[error("The note deleted successfully, but no split occured.")]
    NoSplit,
}

/// the entire note
#[derive(Debug, Clone)]
pub struct Note {
    /// the duration of the first transition
    /// Invariants:
    /// 	1) fade_in_duration is positive
    /// 	2) partials[0].start - fade_in_duration >= 0.0
    fade_in_duration: BeatUnits,

    /// the initial value of fade in
    /// Invariants:
    /// 	1) if fade_in_duration == 0, this is the same as the first partial's pitch
    fade_in_pitch: DetunedPitch,

    /// the duration of the last transition
    /// Invariants:
    /// 	1) fade_out_duration is positive
    fade_out_duration: BeatUnits,

    /// the initial value of fade out
    /// 	1) if fade_out_duration == 0, this is the same as the last partial's pitch
    fade_out_pitch: DetunedPitch,

    /// the partial notes
    /// Invariants:
    /// 	1) any two partials do not overlap at more than just a single point
    /// 	2) partials are kept in order of increasing start time
    /// 	3) there is at least 1 partial in this note at any given time
    partials: Vec<Box<NotePartial>>,

    /// the transitions between each partial
    /// note that there are also transitions into the first and out of the last partial
    /// for partials[i]
    /// Invariants:
    /// 	1) partials[i] has its starting transition at transitions[i]
    /// 	2) partials[i] has its ending transition at transitions[i + 1]
    /// 	3) there are partials.len() + 1 transitions at any given time
    transitions: Vec<CurveShape>,
}

impl Note {
    pub fn new(pitch: DetunedPitch, start: BeatUnits, duration: BeatUnits) -> Self {
        Self {
            fade_in_duration: BeatUnits(0),
            fade_in_pitch: pitch,
            fade_out_duration: BeatUnits(0),
            fade_out_pitch: pitch,
            partials: vec![Box::new(NotePartial::new(pitch, start, duration))],
            transitions: vec![CurveShape::LINEAR, CurveShape::LINEAR],
        }
    }

    /// gets the start time of the note in millibeats
    pub fn start_time(&self) -> BeatUnits {
        self.partials[0].start - self.fade_in_duration
    }

    /// gets the end time of the note in beats
    pub fn end_time(&self) -> BeatUnits {
        let last = self.partials.last().unwrap();
        last.start + last.duration + self.fade_out_duration
    }

    /// gets the duration of the note in beats
    pub fn duration(&self) -> BeatUnits {
        self.end_time() - self.start_time()
    }

    /// returns true if this note is playing at the given time
    pub fn contains_time(&self, time: f64) -> bool {
        self.start_time().into_beats() <= time && time < self.end_time().into_beats()
    }

    /// gets the number of partial notes
    pub fn num_partials(&self) -> usize {
        self.partials.len()
    }

    /// gets the number of transitions
    pub fn num_transitions(&self) -> usize {
        self.partials.len() + 1
    }

    /// gets the number of cents away from a4 at the given time
    pub fn get_cent_delta_a4(&self, time: f64) -> Option<f64> {
        if self.contains_time(time) {
            match self.time_index(time) {
                Ok(i) => {
                    Some(self.partials[i].get_cent_delta_a4(time).unwrap())
                }

                Err(i) => {
                    // should be guaranteed that we get Some(_)
                    Some(self.get_transition(i).unwrap().get_cent_delta_a4(time))
                }
            }
        } else {
            None
        }
    }

    /// gets the index of the note partial or transition at the given time
    /// if out of bounds, gets the index of the nearest transition
    /// Ok(i) -> look at partials[i]
    /// Err(i) -> look at transitions[i]
    fn time_index(&self, time: f64) -> Result<usize, usize> {
        self.partials.binary_search_by(|p| {
            if p.start_time().into_beats() <= time && time < p.end_time().into_beats() {
                Ordering::Equal
            } else if p.start_time().into_beats() <= time {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        })
    }

    /// gets the transition shape at the given index
    /// the transition at index i corresponds to:
    /// 	1) the transition into partial i
    /// 	2) the transition out of partial i - 1
    ///
    /// fails if the transition is out of bounds
    pub fn get_transition_shape(&self, index: usize) -> Option<CurveShape> {
        self.transitions.get(index).copied()
    }

    /// sets the transition shape at the given index
    /// the transition at index i corresponds to:
    /// 	1) the transition into partial i
    /// 	2) the transition out of partial i - 1
    ///
    /// returns the shape that the transition originally had upon success
    pub fn set_transition_shape(&mut self, index: usize, shape: CurveShape) -> Option<CurveShape> {
        if let Some(old_shape) = self.transitions.get_mut(index) {
            let out = Some(*old_shape);
            *old_shape = shape;
            out
        } else {
            None
        }
    }

    /// gets the transition at the given index
    /// the transition at index i corresponds to:
    /// 	1) the transition into partial i
    /// 	2) the transition out of partial i - 1
    /// 
    /// fails if the transition has a duration of 0 or the index is out of bounds
    ///
    /// note that this method may fail even when get_transition_shape does not,
    /// as get_transition_shape will succeed even on transitions with 0 duration
    /// 
    pub fn get_transition(&self, index: usize) -> Option<NoteTransition> {
        if index == 0 {
            if self.fade_in_duration == BeatUnits(0) {
                None
            } else {
                let first_partial = self.partials.first().unwrap();
                Some(NoteTransition {
                    shape: self.transitions[index],
                    start_time: self.start_time(),
                    end_time: first_partial.start_time(),
                    start_pitch: self.fade_in_pitch,
                    end_pitch: first_partial.pitch,
                })
            }
        } else if index == self.transitions.len() - 1 {
            if self.fade_out_duration == BeatUnits(0) {
                None
            } else {
                let last_partial = self.partials.last().unwrap();
                Some(NoteTransition {
                    shape: self.transitions[index],
                    start_time: last_partial.end_time(),
                    end_time: self.end_time(),
                    start_pitch: last_partial.pitch,
                    end_pitch: self.fade_out_pitch,
                })
            }

        } else if index < self.transitions.len() {
            let start_partial = &self.partials[index - 1];
            let end_partial = &self.partials[index];
            Some(NoteTransition {
                shape: self.transitions[index],
                start_time: start_partial.end_time(),
                end_time: end_partial.start_time(),
                start_pitch: start_partial.pitch,
                end_pitch: end_partial.pitch
            })

        } else {
            None
        }
    }

    /// checks if this note overlaps with another note
    pub fn overlaps(&self, other: &Note) -> bool {
        self.start_time() <= other.end_time() && other.start_time() <= self.end_time()
    }
    
    /// checks if this note overlaps with another note at more than just one point
    /// ex (0, 1) and (1, 2) do not over lap while (0, 2) and (1, 3) do
    pub fn overlaps_allow_point(&self, other: &Note) -> bool {
        self.start_time() < other.end_time() && other.start_time() < self.end_time()
    }

    /// attempts to delete the partial with the given index
    ///
    /// it may be that removing the partial splits the note in two,
    /// in which case we will return Ok(note) where the returned note is the
    /// later of the two pieces created
    ///
    /// note that NoteDeletionError doen't necessarily represent an error in deletion,
    /// it might be that deletion was successful, but no split occured
    ///
    pub fn delete_partial(&mut self, index: usize) -> Result<Note, DeleteNotePartialError> {
        self.delete_range(index, index)
    }

    /// attempts to delete the partials with the given index range
    ///
    /// it may be that removing the range splits the note in two,
    /// in which case we will return Ok(note) where the returned note is the
    /// later of the two pieces created
    ///
    /// note that NoteDeletionError doen't necessarily represent a failure to delete.
    /// it might be that deletion was successful, but no split occured
    ///
    pub fn delete_range(&mut self, start: usize, end: usize) -> Result<Note, DeleteNotePartialError> {
        if end < start {
            return Err(DeleteNotePartialError::InvalidRange);
        }
        if end >= self.partials.len() {
            return Err(DeleteNotePartialError::PartialOutOfBounds);
        }
        if end - start + 1 >= self.partials.len() - 1 {
            return Err(DeleteNotePartialError::FullClearError);
        }

        if start == 0 {
            // deleting consecutive starting notes

            // delete notes in range
            self.partials.drain(start..=end);
            self.transitions.drain(start..=end);

            // clear fade in effects
            self.fade_in_duration = BeatUnits(0);
            self.fade_in_pitch = self.partials[0].pitch;

            Err(DeleteNotePartialError::NoSplit)

        } else if end == self.partials.len() - 1 {
            // deleting consecutive ending notes

            // delete notes in range
            self.partials.drain(start..=end);
            self.transitions.drain(start + 1..=end + 1);

            // clear fade out effects
            self.fade_out_duration = BeatUnits(0);
            self.fade_out_pitch = self.partials.last().unwrap().pitch;

            Err(DeleteNotePartialError::NoSplit)

        } else {
            // deleting internal notes

            // perform split
            let last_half = self.split_before_partial(end + 1).unwrap();

            // perform deletion on easy case (delete ending notes)
            let _ = self.delete_range(start, end);

            Ok(last_half)
        }
    }

    /// attempts to split the note between partials before the given partial index
    ///
    /// may fail if we attempt to split before the first partial
    ///
    /// otherwise, returns Some(note) where the returned note is the later of the two halves
	pub fn split_before_partial(&mut self, index: usize) -> Option<Note> {
        if index == 0 || index >= self.partials.len() {
            return None;
        }

        let other_partials = self.partials.split_off(index);
        let other_transitions = self.transitions.split_off(index + 1);
        self.transitions.push(other_transitions[0]);

        let other_fade_in_pitch = other_partials[0].pitch;

        let other = Some(Note {
            partials: other_partials,
            transitions: other_transitions,
            fade_in_duration: BeatUnits(0),
            fade_in_pitch: other_fade_in_pitch,
            fade_out_duration: self.fade_out_duration,
            fade_out_pitch: self.fade_out_pitch,
        });

        self.fade_out_duration = BeatUnits(0);
        self.fade_out_pitch = self.partials.last().unwrap().pitch;

        other
    }

    /// attempts to split the partial at the given time
    ///
    /// returns the note that is the later half of the split on success
    ///
    /// fails if we attempt to split outside of the time we are playing
    /// or at the exact start or end of a partial
    /// 
    pub fn split_at_time(&mut self, time: f64) -> Option<Note> {
        let _ = time;
        todo!()
    }

    /// attempts to combine the given notes
    ///
    /// if use_this_transition is true, then we will use the end point transition
    /// of this note to connect with the other note. if false, the other note's transition
    /// will be used
    ///
    /// (debug build) panics if the notes overlap
    pub fn combine_notes(&mut self, mut other: Note, use_this_transition: bool) {
        debug_assert!(self.overlaps_allow_point(&other), "You may not combine notes that overlap.");

        if self.start_time() < other.start_time() {
            if use_this_transition {
                other.transitions.remove(0);
            } else {
                self.transitions.pop();
            }

            self.partials.append(&mut other.partials);
            self.transitions.append(&mut other.transitions);

            self.fade_out_duration = other.fade_out_duration;
            self.fade_out_pitch = other.fade_out_pitch;
        } else {
            if use_this_transition {
                other.transitions.pop();
            } else {
                self.transitions.remove(0);
            }

            std::mem::swap(&mut self.partials, &mut other.partials);
            self.partials.append(&mut other.partials);

            std::mem::swap(&mut self.transitions, &mut other.transitions);
            self.transitions.append(&mut other.transitions);

            self.fade_in_duration = other.fade_in_duration;
            self.fade_in_pitch = other.fade_in_pitch;
        }
    }

    /// gets a partial immutably
    pub fn get_partial(&self, index: usize) -> Option<&Box<NotePartial>> {
        self.partials.get(index)
    }

    /// gets a partial mutably within a closure, then ensures that it is within
    /// valid bounds by changing its start time, then duration
    /// fails if index is out of bounds, returning false
    pub fn get_partial_mut(&mut self, index: usize, f: impl Fn(&mut NotePartial)) -> bool {
        if index >= self.partials.len() {
            return false;
        }

        let (min_time, max_time) = self.get_partial_bounds(index).unwrap();

        let mut partial = self.partials.get_mut(index).unwrap();
        f(&mut partial);

        if partial.start_time() < min_time {
            partial.set_start_time(min_time);
        }
        if partial.end_time() > max_time {
            partial.set_end_time(max_time);
        }

        true
    }

    /// gets the absolute time bounds for if you were to modify note timeing
    /// fails if the index is out of bounds
    pub fn get_partial_bounds(&self, index: usize) -> Option<(BeatUnits, BeatUnits)> {
        self.get_range_bounds(index, index)
    }

    /// gets the absolute time bounds for if you were to modify note timeing within the given range
    /// fails if the range is out of bounds
    pub fn get_range_bounds(&self, start: usize, end: usize) -> Option<(BeatUnits, BeatUnits)> {
        if start > end || end >= self.partials.len() {
            return None;
        }

        let lower_bound = if start == 0 {
            BeatUnits(0)
        } else {
            self.partials[start - 1].end_time()
        };

        let upper_bound = if end == self.partials.len() - 1 {
            BeatUnits::MAX
        } else {
           self.partials[end + 1].start_time()
        };

        Some((lower_bound, upper_bound))
    }

    /// gets the bounds for if you were to shift the partial by an amount
    /// fails if the index is out of bounds
    pub fn get_partial_shift_bounds(&self, index: usize) -> Option<(BeatUnits, BeatUnits)> {
        self.get_range_shift_bounds(index, index)
    }

    /// gets the bounds for if you were to shift partials within the given range by an amount
    /// fails if the range is out of bounds
    pub fn get_range_shift_bounds(&self, start: usize, end: usize) -> Option<(BeatUnits, BeatUnits)> {
        if start > end || end >= self.partials.len() {
            return None;
        }

        let lower_bound = if start == 0 {
            -self.partials[start].start_time()
        } else {
            self.partials[start - 1].end_time() - self.partials[start].start_time()
        };

        let upper_bound = if end == self.partials.len() - 1 {
            BeatUnits::MAX
        } else {
           self.partials[end + 1].start_time() - self.partials[end].end_time()
        };

        Some((lower_bound, upper_bound))

    }

    pub fn partial_index_iter<'a>(&'a self) -> impl Iterator<Item = usize> + 'a {
        (0..self.partials.len()).into_iter()
    }

    pub fn transition_index_iter<'a>(&'a self) -> impl Iterator<Item = usize> + 'a {
        (0..self.transitions.len()).into_iter()
    }

    pub fn transition_iter<'a>(&'a self) -> NoteTransitionIter<'a> {
        NoteTransitionIter { note: self, index: 0 }
    }

    pub fn partial_iter<'a>(&'a self) -> impl Iterator<Item = &'a NotePartial> + 'a {
        self.partials.iter().map(|i| &**i)
    }

}

#[derive(Debug)]
pub struct NoteTransitionIter<'a> {
    note: &'a Note,
    index: usize
}

impl<'a> Iterator for NoteTransitionIter<'a> {
    type Item = NoteTransition;

    fn next(&mut self) -> Option<Self::Item> {
        while self.note.get_transition(self.index).is_none() {
            self.index += 1;
            if self.index >= self.note.num_transitions() {
                return None;
            }
        }
        let out = self.note.get_transition(self.index);
        self.index += 1;
        out
    }
}

/// a part of a note with a constant base pitch and possible vibrato
#[derive(Debug, Clone)]
pub struct NotePartial {
    /// the pitch of the partial note
    pub pitch: DetunedPitch,

    /// the time when the partial starts in millibeats
    /// Invariants:
    /// 	1) must be greater than 0
    start: BeatUnits,

    /// the duration of the partial in millibeats
    /// Invariants:
    /// 	1) must be greater than MIN_DURATION
    duration: BeatUnits,

    /// the vibrato of the note
    /// Invariants:
    ///  1) vibrato is contained entirely wiithin the note's up time
    vibrato: Vibrato,
}

impl NotePartial {
    pub const MIN_DURATION: BeatUnits = BeatUnits(1);

    pub fn new(pitch: DetunedPitch, start: BeatUnits, duration: BeatUnits) -> Self {
        Self {
            pitch,
            start,
            duration: duration.max(Self::MIN_DURATION),
            vibrato: Vibrato::new(),
        }
    }

    pub fn start_time(&self) -> BeatUnits {
        self.start
    }

    pub fn end_time(&self) -> BeatUnits {
        self.start + self.duration
    }

    pub fn duration(&self) -> BeatUnits {
        self.duration
    }

    pub fn vibrato(&self) -> &Vibrato {
        &self.vibrato
    }

    /// allows mutable access the contained vibrato via a closure
    /// after the closure executes, ensures that the vibrato maintains the invariants
    /// of a partial note
    pub fn vibrato_mut(&mut self, f: impl Fn(&mut Vibrato)) {
        f(&mut self.vibrato);
        if self.vibrato.start_time() < self.start_time() {
            self.vibrato.set_start_time(self.start_time());
        }
        if self.vibrato.end_time() > self.end_time() {
            self.vibrato.set_end_time(self.end_time());
        }
    }

    /// sets the start time of the vibrato
    /// does not let start time go below 0.0
    /// moves vibrato so it has the same relative time as before the operation
    pub fn set_start_time(&mut self, time: BeatUnits) {
        let delta = time.max(BeatUnits(0)) - self.start;
        self.start = time.max(BeatUnits(0));
        self.vibrato.set_start_time(self.vibrato.start_time() + delta);
    }

    /// sets the end time of the vibrato
    /// does not let end time go below start time
    pub fn set_end_time(&mut self, time: BeatUnits) {
        self.set_duration(time - self.start_time());
    }

    /// sets the duration of the vibrato
    /// does not let duration go below Self::MIN_DURATION
    /// modifies vibrato to not exceed the bounds of the note
    pub fn set_duration(&mut self, time: BeatUnits) {
        self.duration = time.max(Self::MIN_DURATION);
        if self.end_time() < self.vibrato.end_time() {
            self.vibrato.set_end_time(self.end_time());
        }
    }

    /// fails if the note is not active at the given time
    pub fn get_cent_delta_a4(&self, time: f64) -> Option<f64> {
        if !self.contains_time(time) {
            None
        } else {
            Some(self.vibrato.get_cent_delta(time) + self.pitch.cent_delta_a4() as f64)
        }
    }

    /// returns true if the given time is contained in the note
    pub fn contains_time(&self, time: f64) -> bool {
        self.start_time().into_beats() <= time && time < self.end_time().into_beats()
    }
}

/// a description of vibrato for a length of time
#[derive(Debug, Clone)]
pub struct Vibrato {
    /// the time when vibrato starts in millibeats
    /// Invariants:
    /// 	1) must be above or equal to 0
    start: BeatUnits,

    /// the duration of the vibrato in millibeats
    /// Invariants:
    /// 	1) must be above or equal to 0
    duration: BeatUnits,

    /// phase as a fraction of wavelength
    pub phase: f32,

    /// frequency as a multiple of beats
    /// Invariants:
    /// 	1) must be greater than 0.0
    freq: f32,

    /// fade in duration in millibeats
    /// Invariants:
    /// 	1) must be within 0 and duration
    fade_in_duration: BeatUnits,

    /// fade out duration in millibeats
    /// Invariants:
    /// 	1) must be within 0 and duration
    fade_out_duration: BeatUnits,

    /// the curve for fade in amplitude
    pub fade_in_shape: CurveShape,

    /// the curve for fade out amplitude
    pub fade_out_shape: CurveShape,

    /// amplitude in cents
    pub amplitude: u32,
}

impl Vibrato {
    /// creates a new empty vibrato
    pub fn new() -> Self {
        Self {
            start: BeatUnits(0),
            duration: BeatUnits(0),
            phase: 0.0,
            freq: 0.5,
            fade_in_duration: BeatUnits(0),
            fade_out_duration: BeatUnits(0),
            fade_in_shape: CurveShape::LINEAR,
            fade_out_shape: CurveShape::LINEAR,
            amplitude: 50,
        }
    }

    /// gets the start time of the vibrato
    pub fn start_time(&self) -> BeatUnits {
        self.start
    }

    /// gets the end time of the vibrato
    pub fn end_time(&self) -> BeatUnits {
        self.start + self.duration
    }

    /// gets the duration of the vibrato
    pub fn duration(&self) -> BeatUnits {
        self.duration
    }

    /// gets the fade in duration of the vibrato
    pub fn fade_in_duration(&self) -> BeatUnits {
        self.fade_in_duration
    }

    /// gets the fade out duration of the vibrato
    pub fn fade_out_duration(&self) -> BeatUnits {
        self.fade_out_duration
    }

    /// sets the start time of the vibrato
    /// does not let start time go below 0
    pub fn set_start_time(&mut self, time: BeatUnits) {
        self.start = time.max(BeatUnits(0));
    }

    /// sets the end time of the vibrato
    /// does not let end time go below start time
    pub fn set_end_time(&mut self, time: BeatUnits) {
        self.set_duration(time - self.start_time());
    }

    /// sets the duration of the vibrato
    /// does not let duration go below 0.0
    /// changes fade in/out durations if this would lead to them being
    /// greater than the actual duration
    pub fn set_duration(&mut self, time: BeatUnits) {
        self.duration = time.max(BeatUnits(0));
        self.fade_in_duration = self.fade_in_duration.min(self.duration());
        self.fade_out_duration = self.fade_out_duration.min(self.duration());
    }

    /// sets the fade in duration of the vibrato
    /// will be clamped to 0.0, duration
    pub fn set_fade_in_duration(&mut self, duration: BeatUnits) {
        self.fade_in_duration = duration.clamp(BeatUnits(0), self.duration());
    }

    /// sets the fade out duration of the vibrato
    /// will be clamped to 0.0, duration
    pub fn set_fade_out_duration(&mut self, duration: BeatUnits) {
        self.fade_out_duration = duration.clamp(BeatUnits(0), self.duration());
    }

    /// gets the amount of modulation (should be added to base frequency) in cents
    pub fn get_cent_delta(&self, time: f64) -> f64 {
        if time < self.start_time().into_beats() || time > self.end_time().into_beats() {
            return 0.0
        }

        let fundamental = f64::sin(f64::consts::TAU * (self.freq as f64 * time - self.phase as f64));

        let left_envelope = if self.fade_in_duration <= BeatUnits(0) {
            // required envelope
            let denominator = (1.0 + 2.0 * (self.phase as f64 - self.start.into_beats() * self.freq as f64)) % 1.0;
            if denominator.is_finite() {
                (2.0 * self.freq as f64 * (time - self.start.into_beats()) / denominator).min(1.0)
            } else {
                1.0
            }
        } else {
            // custom envelope
            self.fade_in_shape.interpolate(
                time,
                self.start_time().into_beats(),
                (self.start_time() + self.fade_in_duration).into_beats(),
                0.0,
                1.0
            )
        };

        let right_envelope = if self.fade_out_duration <= BeatUnits(0) {
            // required envelope
            let denominator = ( 2.0 * (self.freq as f64 * self.end_time().into_beats() - self.phase as f64 ) ) % 1.0;
            if denominator.is_finite() {
                (2.0 * self.freq as f64 * (self.end_time().into_beats() - time) / denominator).min(1.0)
            } else {
                1.0
            }
        } else {
            // custom envelope
            self.fade_out_shape.interpolate(
                time,
                (self.end_time() - self.fade_out_duration).into_beats(),
                self.end_time().into_beats(),
                1.0,
                0.0
            )
        };

        right_envelope * left_envelope * self.amplitude as f64 * fundamental
    }

}

