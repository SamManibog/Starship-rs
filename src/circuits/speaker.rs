use crate::circuit::{CircuitBuilder, Circuit, ConnectionSpecification};

#[derive(Debug, Clone)]
pub struct SpeakerBuilder {
}

impl SpeakerBuilder {
    const SPECIFICATION: ConnectionSpecification = ConnectionSpecification {
        name: "Speaker",
        output_names: &[],
        input_names: &["In"],
        size: egui::vec2(100.0, 100.0),
    };

    pub fn new() -> Self {
        Self{ }
    }
}

impl CircuitBuilder for SpeakerBuilder {
    fn specification(&self) -> &'static ConnectionSpecification {
        &Self::SPECIFICATION
    }

    fn build(&self) -> Box<dyn Circuit> {
        panic!("Speakers cannot be directly built.");
    }

    fn request_size(&self) -> Option<egui::Vec2> {
        Some(egui::vec2(100.0, 70.0))
    }
}
