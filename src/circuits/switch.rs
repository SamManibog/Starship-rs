use crate::circuit::{CircuitBuilder, Circuit, ConnectionSpecification};

#[derive(Debug, Clone)]
pub struct SwitchBuilder {
    value: f32,
    current_text: String
}

impl SwitchBuilder {
    const SPECIFICATION: ConnectionSpecification = ConnectionSpecification {
        name: "Switch",
        output_names: &["Out"],
        input_names: &["In"],
        size: egui::vec2(100.0, 100.0),
    };

    pub fn new() -> Self {
        let value = 0.0_f32;
        Self{
            value,
            current_text: value.to_string()
        }
    }
}

impl CircuitBuilder for SwitchBuilder {
    fn show(&mut self, ui: &mut egui::Ui) {
        todo!()
    }

    fn specification(&self) -> &'static ConnectionSpecification {
        &Self::SPECIFICATION
    }

    fn build(&self) -> Box<dyn Circuit> {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
enum SwitchType {
    Toggle,
    PressAndHold,
    OneShot(f32)
}
