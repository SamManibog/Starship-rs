/// Sequencer for scheduling binary on-off values for drums (open or close)
pub mod drum_sequencer;

/// Sequencer for scheduling Midi values
pub mod piano_sequencer;

/// Sequencer for scheduling automations for piano and drums (submenu of drum/piano sequencer)
pub mod automation_sequencer;

/// Sequencer combining all audio
pub mod track_sequencer;

/// transition curves for non-note inputs
pub mod curve;

/// curves for note inputs
pub mod note;
