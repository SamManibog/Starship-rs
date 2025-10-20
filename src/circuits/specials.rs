use crate::circuit::{BuildState, Circuit, CircuitBuilder, CircuitSpecification};

#[derive(Debug, Clone)]
pub struct SpecialInputBuilder {
    name: String
}

impl SpecialInputBuilder {
    pub const SPECIFICATION: CircuitSpecification = CircuitSpecification {
        output_names: &[],
        input_names: &["Out"],
        size: egui::vec2(100.0, 100.0),
        playback_size: None,
    };

    pub fn new(name: String) -> Self {
        Self{
            name
        }
    }
}

impl CircuitBuilder for SpecialInputBuilder {
    fn name(&self) -> &str {
        &self.name
    }

    fn specification(&self) -> &'static CircuitSpecification {
        &Self::SPECIFICATION
    }

    fn build(&self, _: &BuildState) -> Box<dyn Circuit> {
        panic!("Special inputs cannot be directly built.");
    }

    fn request_size(&self) -> Option<egui::Vec2> {
        Some(egui::vec2(100.0, 70.0))
    }
}

#[derive(Debug, Clone)]
pub struct SpecialOutputBuilder {
    name: String
}

impl SpecialOutputBuilder {
    pub const SPECIFICATION: CircuitSpecification = CircuitSpecification {
        output_names: &[],
        input_names: &["In"],
        size: egui::vec2(100.0, 100.0),
        playback_size: None,
    };

    pub fn new(name: String) -> Self {
        Self{
            name
        }
    }
}

impl CircuitBuilder for SpecialOutputBuilder {
    fn name(&self) -> &str {
        &self.name
    }

    fn specification(&self) -> &'static CircuitSpecification {
        &Self::SPECIFICATION
    }

    fn build(&self, _: &BuildState) -> Box<dyn Circuit> {
        panic!("Special outputs cannot be directly built.");
    }

    fn request_size(&self) -> Option<egui::Vec2> {
        Some(egui::vec2(100.0, 70.0))
    }
}
