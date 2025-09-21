use crate::circuit::{CircuitBuilder, Circuit, CircuitSpecification};

#[derive(Debug, Clone)]
pub struct ConstantBuilder {
    value: f32,
    current_text: String
}

impl ConstantBuilder {
    const SPECIFICATION: CircuitSpecification = CircuitSpecification {
        name: "Constant",
        output_names: &["out"],
        input_names: &[],
    };

    pub fn new() -> Self {
        let value = 0.0_f32;
        Self{
            value,
            current_text: value.to_string()
        }
    }
}

impl CircuitBuilder for ConstantBuilder {
    fn show(&mut self, ui: &mut egui::Ui) {
        let mut text = self.current_text.clone();
        let response = ui.text_edit_singleline(&mut text);
        if response.changed() {
            //ensure entered characters are valid in a float
            if text.find(|char: char| {
                !char.is_numeric() && char != '.' && char != '-'
            }) == None {
                self.current_text = text;
            }
        }

        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if let Ok(value) = self.current_text.parse::<f32>() {
                self.value = value;
            }
            self.current_text = self.value.to_string();
        }
    }

    fn specification(&self) -> &'static CircuitSpecification {
        &Self::SPECIFICATION
    }

    fn build(&self) -> Box<dyn Circuit> {
        Box::new(Constant{ value: self.value })
    }
}

#[derive(Debug, Default)]
pub struct Constant {
    value: f32
}

impl Circuit for Constant {
    fn operate(&mut self, inputs: &[f32], outputs: &mut[f32]) {
        let _ = inputs;
        outputs[0] = self.value;
    }
}
