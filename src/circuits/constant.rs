use crate::{circuit::{BuildState, Circuit, CircuitBuilder, CircuitSpecification}, utils::PitchOrValue};

#[derive(Debug, Clone)]
pub struct ConstantBuilder {
    value: PitchOrValue<f32>,
    text: String
}

impl ConstantBuilder {
    const SPECIFICATION: CircuitSpecification = CircuitSpecification {
        output_names: &["Out"],
        input_names: &[],
        size: egui::vec2(100.0, 100.0),
        playback_size: None,
    };

    const NAME: &'static str = "Constant";

    pub fn new() -> Self {
        let value = PitchOrValue::Value(0.0);
        Self {
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
        crate::utils::pitch_or_value_input(ui, &mut self.text, &mut self.value);
    }

    fn specification(&self) -> &'static CircuitSpecification {
        &Self::SPECIFICATION
    }

    fn build(&self, state: &BuildState) -> Box<dyn Circuit> {
        let value = match self.value {
            PitchOrValue::Value(val) => val,
            PitchOrValue::Pitch(pitch) => {
                state.tuning.get_pitch_frequency(&pitch)
            }
        };
        Box::new(Constant{ value })
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
