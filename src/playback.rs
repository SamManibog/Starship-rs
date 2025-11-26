use std::collections::HashMap;

use crate::{live_plugin_id::LivePluginId, plugin_graph::{EffectGraph, PlaybackOrder}};

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
    synths: HashMap<LivePluginId, Box<ComponentMetadata<LiveSynthContainer>>>,
    drums: HashMap<LivePluginId, Box<ComponentMetadata<LiveDrumContainer>>>,
    effects: HashMap<LivePluginId, Box<ComponentMetadata<LiveEffectContainer>>>,

    /// a map from an id to an effect group's graph and main output
    effect_group_outputs: HashMap<LivePluginId, (Box<EffectGraph>, *mut LiveEffectContainer)>,

    /// the main output
    main_output: *mut LiveEffectContainer,

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
}

