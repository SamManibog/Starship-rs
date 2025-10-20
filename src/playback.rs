pub trait LiveComponent {
    /// reset any internal state when playback stops, for instance
    fn reset(&mut self);

    /// gets a list of automatable components.
    fn get_automatable(&self) -> &[AutomationSpecification];

    /// sets the read value for an automatable component.
    fn set_automatable(&mut self, index: usize, value: f32);
}

pub(crate) trait LiveComponentUtils {
    /// sets the value for an automatable component, clamping the value to the allowed range
    fn set_automatable_safe(&mut self, index: usize, value: f32);
}

impl<T> LiveComponentUtils for T where T: LiveComponent {
    fn set_automatable_safe(&mut self, index: usize, value: f32) {
        let automatable = self.get_automatable();
        assert!(index <= automatable.len(), "Attempted to set automation for non-existant automatable.");
        let range = automatable[index].range;
        self.set_automatable(
            index,
            value.clamp(range.0, range.1)
        );
    }
}

pub trait LiveEffect: LiveComponent {
    fn update(&mut self, sample: f32, sample_rate: u32) -> f32;
}

pub trait LiveDrum: LiveComponent {
    fn update(&mut self, hit: bool, sample_rate: u32) -> f32;
}

pub trait SynthPlugin: LiveComponent {
    /// The number of voices allowed on this plugin
    fn voice_count(&self) -> usize;

    fn update(&mut self, voices: &[VoiceState], sample_rate: u32) -> f32;
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
    /// The name of the automatable input
    pub name: String,

	/// The allowed range of values for the automatable input
    /// Input values are clamped during runtime.
    pub range: (f32, f32),
}

