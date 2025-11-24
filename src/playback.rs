use std::collections::HashMap;

use crate::{live_plugin_id::{LivePluginId, LivePluginIdManager}, plugin_graph::PlaybackOrder};

pub struct PlaybackState {
    /// manager for universal id of components
    live_id_manager: LivePluginIdManager,

    /// map from an identifier to important data regarding the component
    synths: HashMap<LivePluginId, Box<ComponentMetadata<LiveSynthContainer>>>,
    drums: HashMap<LivePluginId, Box<ComponentMetadata<LiveDrumContainer>>>,
    effects: HashMap<LivePluginId, Box<ComponentMetadata<LiveEffectContainer>>>,

    order: PlaybackOrder,
}

#[derive(Debug)]
pub struct ComponentMetadata<T> {
    /// a pointer to the data of the component
    pub component: *mut T,

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

    /// gets a list of automatable components.
    fn get_automatable(&self) -> &[AutomationSpecification];
}

pub trait LiveEffect: LivePlugin {
    fn update(&mut self, sample: f32, automations: &AutomationState, sample_rate: u32) -> f32;
}

pub trait LiveDrum: LivePlugin {
    fn update(&mut self, state: DrumState, automations: &AutomationState, sample_rate: u32) -> f32;
}

pub trait LiveSynth: LivePlugin {
    /// The number of voices allowed on this plugin
    fn voice_count(&self) -> usize;

    fn update(&mut self, voices: &[VoiceState], automations: &AutomationState, sample_rate: u32) -> f32;
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
pub struct AutomationSpecification {
    /// The internal id of the automation should be unique within the same component
    /// In practice, these correspond to an index in an array so its best to keep these
    /// low as possible so as to avoid excessive memory cost
    pub id: usize,

    /// The displayed name of the automatable input
    pub name: String,

	/// The allowed range of values for the automatable input
    /// Input values are clamped during runtime.
    pub range: (f32, f32),
}

#[derive(Debug, Clone)]
pub struct AutomationState<'a> {
    map: &'a [f32],
}

impl<'a> AutomationState<'a> {
    pub fn new(data: &'a [f32]) -> Self {
        Self {
            map: data
        }
    }
    
    pub fn query(&self, id: usize) -> f32 {
        self.map.get(id).copied().unwrap_or(0.0)
    }
}

pub struct LiveSynthContainer {
    synth: Box<dyn LiveSynth>,
    automations: Vec<f32>,
    voices: Vec<VoiceState>
}

impl LiveSynthContainer {
    pub fn update(&mut self, sample_rate: u32) -> f32 {
        self.synth.update(&self.voices, &AutomationState::new(&self.automations), sample_rate)
    }
}

pub struct LiveDrumContainer {
    drum: Box<dyn LiveDrum>,
    automations: Vec<f32>,
    state: DrumState
}

impl LiveDrumContainer {
    pub fn update(&mut self, sample_rate: u32) -> f32 {
        self.drum.update(self.state, &AutomationState::new(&self.automations), sample_rate)
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
        let automation_count = effect.get_automatable().len();
        Self {
            effect,
            automations: vec![0.0; automation_count],
            sample: 0.0,
            buffered_sample: 0.0,
        }
    }

    pub fn update(&mut self, sample_rate: u32) -> f32 {
        let out = self.effect.update(self.sample, &AutomationState::new(&self.automations), sample_rate);
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

    /*
    pub fn remove_send(&mut self, target: *mut Self) {
        if let Ok(index) = self.sends.binary_search(&target) {
            self.sends.remove(index);
        }
    }

    pub fn add_send(&mut self, target: *mut Self) {
        if let Err(index) = self.sends.binary_search(&target) {
            self.sends.insert(index, target);
        }
    }
    */
}

