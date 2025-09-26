use crate::circuit::{CircuitBuilder, Circuit, ConnectionSpecification};

#[derive(Debug, Clone)]
pub struct ConstantBuilder {
    value: f32,
    text: String
}

impl ConstantBuilder {
    const SPECIFICATION: ConnectionSpecification = ConnectionSpecification {
        output_names: &["Out"],
        input_names: &[],
        size: egui::vec2(100.0, 100.0),
    };

    const NAME: &'static str = "Constant";

    pub fn new() -> Self {
        let value = 0.0_f32;
        Self{
            value,
            text: value.to_string()
        }
    }
}

impl CircuitBuilder for ConstantBuilder {
    fn name(&self) -> &str {
        Self::NAME
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        crate::utils::float_input(ui, &mut self.text, &mut self.value);
    }

    fn specification(&self) -> &'static ConnectionSpecification {
        &Self::SPECIFICATION
    }

    fn build(&self) -> Box<dyn Circuit> {
        Box::new(Constant{ value: self.value })
    }

    fn request_size(&self) -> Option<egui::Vec2> {
        Some(egui::vec2(100.0, 70.0))
    }
}

#[derive(Debug, Default)]
pub struct Constant {
    value: f32
}

impl Circuit for Constant {
    fn operate(&mut self, inputs: &[f32], outputs: &mut[f32], _: f32) {
        let _ = inputs;
        outputs[0] = self.value;
    }
}
