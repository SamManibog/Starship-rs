use crate::{pitch::Pitch, live_plugin_id::LivePluginId};

pub struct PianoSequencer {
    /// base data for instruments
    instruments: Vec<InstrumentData>,

    /// extra data for instruments used during playback
    playback_instruments: Vec<InstrumentData>,

    /// whether or not we are in playback mode
    playback: bool,
}

/// The data for playback of one instrument
/// mainly interacted with via the update function,
/// which requires a monotonically increasing clock
struct InstrumentPlaybackBuffer {
    /// The underlying data
    data: *const InstrumentData,

    /// The current index of the note to play next from data.notes
    index: usize,

    /// The notes that are currently playing
    active: Vec<*const NoteData>,

    /// The notes that have been added during playback
    added: Vec<NoteData>,
}

/// The data associated with a single instrument
struct InstrumentData {
    /// A list of notes to play, sorted by start time
    notes: Vec<NoteData>,

    /// The instrument to play the notes on
    instrument: LivePluginId,
}

struct NoteData {
    /// The base pitch of the note
    base_pitch: Pitch,

    /// The offset of the pitch in cents
    detune: f32,

    /// When the note starts in beats (1 qtrnote = 1 beat)
    start: f32,

    /// How long the note lasts in beats (1 qtrnote = 1 beat)
    duration: f32,

    /// Midi velocity of the note
    velocity: u8,
}
