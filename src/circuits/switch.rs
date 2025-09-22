use crate::circuit::{CircuitBuilder, Circuit, CircuitSpecification};

#[derive(Debug, Clone)]
pub struct SwitchBuilder {
    value: f32,
    current_text: String
}

impl SwitchBuilder {
    const SPECIFICATION: CircuitSpecification = CircuitSpecification {
        name: "Switch",
        output_names: &["Out"],
        input_names: &["In"],
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

    fn specification(&self) -> &'static CircuitSpecification {
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
