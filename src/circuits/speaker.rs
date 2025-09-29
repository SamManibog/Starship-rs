use crate::circuit::{BuildState, Circuit, CircuitBuilder, CircuitSpecification};

#[derive(Debug, Clone)]
pub struct SpeakerBuilder {
}

impl SpeakerBuilder {
    const SPECIFICATION: CircuitSpecification = CircuitSpecification {
        output_names: &[],
        input_names: &["In"],
        size: egui::vec2(100.0, 100.0),
        playback_size: None,
    };

    const NAME: &'static str = "Speaker";

    pub fn new() -> Self {
        Self{ }
    }
}

impl CircuitBuilder for SpeakerBuilder {
    fn name(&self) -> &str {
        Self::NAME
    }

    fn specification(&self) -> &'static CircuitSpecification {
        &Self::SPECIFICATION
    }

    fn build(&self, _: &BuildState) -> Box<dyn Circuit> {
        panic!("Speakers cannot be directly built.");
    }

    fn request_size(&self) -> Option<egui::Vec2> {
        Some(egui::vec2(100.0, 70.0))
    }
}
