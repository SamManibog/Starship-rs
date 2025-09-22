use crate::circuit::{CircuitBuilder, Circuit, CircuitSpecification};

#[derive(Debug, Clone)]
pub struct SpeakerBuilder {
}

impl SpeakerBuilder {
    const SPECIFICATION: CircuitSpecification = CircuitSpecification {
        name: "Speaker",
        output_names: &[],
        input_names: &["In"],
    };

    pub fn new() -> Self {
        Self{ }
    }
}

impl CircuitBuilder for SpeakerBuilder {
    fn specification(&self) -> &'static CircuitSpecification {
        &Self::SPECIFICATION
    }

    fn build(&self) -> Box<dyn Circuit> {
        panic!("Speakers cannot be directly built.");
    }

    fn request_size(&self) -> Option<egui::Vec2> {
        Some(egui::vec2(100.0, 70.0))
    }
}
