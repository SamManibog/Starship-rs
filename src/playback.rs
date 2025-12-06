use std::collections::HashMap;

use crate::{live_plugin_id::LivePluginId, pitch::equal_temperment, plugin_graph::{EffectGraph, PlaybackOrder}};

pub type NoteId = u32;
pub type InputId = u32;

#[derive(Debug)]
pub enum PlaybackCommand {
    /// begin playing audio
    StartPlayback,

    /// stop playing audio
    StopPlayback,

    /// add a synthesizer of the given name using the given id
    AddSynth{name: String, id: LivePluginId},

    /// remove the synthesizer with the given id
    RemoveSynth(LivePluginId),

    /// add a drum of the given name using the given id
    AddDrum{name: String, id: LivePluginId},

    /// remove the drum with the given id
    RemoveDrum(LivePluginId),

    /// add an effect of the given name using the given id
    AddEffect{name: String, id: LivePluginId},

    /// remove the effect with the given id
    RemoveEffect(LivePluginId),

    /// connect two effects in an effects group
    ConnectEffects{group: LivePluginId, src: LivePluginId, dst: LivePluginId},

    /// disconnect two effects in an effects group
    DisconnectEffects{group: LivePluginId, src: LivePluginId, dst: LivePluginId},

    /// connect an effect directly to the output of an effects group
    ConnectDirectOutput{group: LivePluginId, src: LivePluginId},

    /// disconnect an effect directly from the output of an effects group
    DisconnectDirectOutput{group: LivePluginId, src: LivePluginId},

    /// connect a synth/drum directly to the output of an effects group
    ConnectDirectInput{group: LivePluginId, src: LivePluginId},

    /// disconnect a synth/drum directly from the output of an effects group
    DisconnectDirectInput{group: LivePluginId, src: LivePluginId},
}

pub struct ComponentFactory {
    synths: Vec<(String, Box<dyn Fn()->Box<dyn LiveSynth>>)>,
    drums: Vec<(String, Box<dyn Fn()->Box<dyn LiveSynth>>)>,
    effects: Vec<(String, Box<dyn Fn()->Box<dyn LiveSynth>>)>,
}

pub struct PlaybackState {
    /// map from an id to important data regarding the component
    synths: HashMap<LivePluginId, Box<ComponentMetadata<*mut dyn LiveSynth>>>,
    drums: HashMap<LivePluginId, Box<ComponentMetadata<*mut dyn LiveDrum>>>,
    effects: HashMap<LivePluginId, Box<ComponentMetadata<*mut LiveEffectContainer>>>,

    /// a map from an id to an effect group's graph and main output
    effect_group_outputs: HashMap<LivePluginId, (Box<EffectGraph>, *mut LiveEffectContainer)>,

    /// the main output
    main_output: *mut LiveEffectContainer,

    /// the id of the main output
    main_output_id: LivePluginId,

    order: PlaybackOrder,
}

#[derive(Debug)]
pub struct ComponentMetadata<T> {
    /// a pointer to the data of the component
    pub component: T,

    /// the effect group that the component belongs to
    pub group: LivePluginId
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct AutomationId {
    pub component: LivePluginId,
    pub local_id: usize
}

pub trait LivePlugin {
    /// reset any internal state when playback stops, for instance
    fn reset(&mut self);

    /// gets a list of secondary inputs
    /// the return value of this must remain constant for the entire runtime
    fn get_inputs(&self) -> Vec<InputSpecification>;

    /// sets the value of a seconcdary input by its id
    /// we guarantee that this function is only called with ids
    /// and values specified by the get_inputs function
    fn set_input(&mut self, id: InputId, value: f64);
}

pub trait LiveEffect: LivePlugin {
    fn update(&mut self, sample: f32, sample_rate: u32) -> f32;
}

pub trait LiveDrum: LivePlugin {
    fn update(&mut self, sample_rate: u32) -> f32;
}

pub trait LiveSynth: LivePlugin {
    /// whether or not this synthesizer allows notes to change frequency
    fn allow_frequency_change(&self) -> bool;

    /// whether or not this synthesizer handles aftertouch
    fn allow_aftertouch(&self) -> bool;

    /// turns on a note.
    /// You must handle id management yourself
    fn set_note_on(&mut self, id: NoteId, freq: f32, velocity: u8);

    /// turns off a note
    fn set_note_off(&mut self, id: NoteId, freq: f32);

    /// sets the frequency of a note
    /// if allow_frequency_change is false, this function will never be called
    fn set_note_freq(&mut self, id: NoteId, freq: f32);

    /// sets the aftertouch for a note
    /// if allow_aftertouch is false, this function will never be called
    fn set_note_aftertouch(&mut self, id: NoteId, aftertouch: f32);

    /// Set the input to the given value.
    /// We guarantee that only ids as specified in the get_inputs() function will be passed as
    /// arguments to this function.
    fn set_input(&mut self, id: InputId, value: f64);

    /// produces a sample
    /// for each sample production cycle, it is guaranteed that this function
    /// is called last.
    fn update(&mut self, sample_rate: u32) -> f32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrumState {
    Hit(u8),
    Hold(u8),
    Off
}

#[derive(Debug, Clone, Copy)]
pub enum VoiceState {
    /// The voice just became active on this sample
    JustActive{freq: f32, vel: u8},

    /// The voice is active, but became active on a previous sample
    Active{freq: f32, vel: u8},

    /// The voice is not active
    Inactive
}

impl Default for VoiceState {
    fn default() -> Self {
        Self::Inactive
    }
}

#[derive(Debug, Clone)]
pub struct InputSpecification {
    /// The id of the input.
    /// These should be unique within the same plugin
    pub id: InputId,

    /// The displayed name of the input
    pub name: String,

    /// A shortened name for the input
    pub short_name: String,

    /// Whether the input should be given as frequencies of notes
    /// this changes the behavior of curves to account for the logarithmic scale
    /// used by tuning systems
    pub is_note_input: bool,

	/// The allowed range of values for the automatable input
    /// Input values are clamped during runtime.
    ///	You must ensure range.0 < range.1, both values must also be real numbers
    pub range: (f64, f64),

    /// the number of allowed values for the input.
    /// values will be passed with an even distribution within the specified range
    /// 0 corresponds to a continuous amount of values (no snapping)
    /// 1 corresponds to snapping between notes (no microtones allowed)
    /// 2 corresponds to two possible input values
    /// 3 corresponds to three possible input values and so on
    ///
    /// Notes:
    /// 	- if is_note_input is true, only 0 or 1 may be used
    /// 	- a value of 1 may only be used if is_note_input is true
    ///
    pub input_values: u32,

    /// The default value of the input
    /// When reset is called on the plugin that produced it,
    /// the input should default to this value
    ///
    /// default should be within self.range
    pub default: f64
}

impl InputSpecification {
    /// whether or not the specification is valid
    pub fn is_valid(&self) -> bool {
        // check that range does not just allow only a single number
        (self.range.0 < self.range.1)

        // check default is within our range
        && (self.range.0 <= self.default && self.default <= self.range.1)

        // check that if is_note_input is true, then input_values is 0 or 1
        && (!self.is_note_input || self.input_values <= 1)

        // check that if input_values is 1, is_note_input is true
        && (self.input_values != 1 || self.is_note_input)
    }

    /// whether or not the input supports continouous values
    pub fn is_continuous(&self) -> bool {
        self.input_values == 0
    }

    /// whether the input requires discrete values
    pub fn is_discrete(&self) -> bool {
        self.input_values > 0
    }

    /// snaps the given value based on...
    /// 	1) self.range
    /// 	2) self.input_values
    /// 	3) self.is_note_input
    /// assumes this specification is valid
    pub fn snap(&self, value: f64) -> f64 {
        let clamped = value.clamp(self.range.0, self.range.1);

        if self.is_continuous() {
            clamped
        } else if self.is_note_input {
            equal_temperment::quantize_semitone(440.0, value)
        } else {
            let range_diff = self.range.1 - self.range.0;
            let steps = (self.input_values - 1) as f64;
            let scale_factor = steps / range_diff;

            //f64::round( steps * (clamped - self.range.0) / range_diff ) * range_diff / steps + self.range.0
            f64::round( scale_factor * (clamped - self.range.0) ) / scale_factor + self.range.0
        }
    }

}

pub struct LiveEffectContainer {
    /// the implementation of the effect
    effect: Box<dyn LiveEffect>,

    /// the buffer of automations to pass to the effect
    automations: Vec<f32>,

    /// the sample to pass to the effect
    sample: f32,

    /// the sample to pass to the effect on the next update
    buffered_sample: f32,
}

impl LiveEffectContainer {
    pub unsafe fn new(effect: Box<dyn LiveEffect>) -> Self {
        let automation_count = effect.get_inputs().len();
        Self {
            effect,
            automations: vec![0.0; automation_count],
            sample: 0.0,
            buffered_sample: 0.0,
        }
    }

    pub fn update(&mut self, sample_rate: u32) -> f32 {
        let out = self.effect.update(self.sample, sample_rate);
        self.sample = self.buffered_sample;
        self.buffered_sample = 0.0;
        out
    }

    pub fn send(&mut self, sample: f32) {
        self.sample += sample;
    }

    pub fn save(&mut self, sample: f32) {
        self.buffered_sample += sample;
    }
}

