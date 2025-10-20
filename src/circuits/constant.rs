use std::{cell::RefCell, rc::Rc};

use crate::{circuit::{BuildState, Circuit, CircuitBuilder, CircuitSpecification}, utils::PitchOrValue};

#[derive(Debug, Clone)]
pub struct ConstantBuilder {
    data: Rc<RefCell<ConstantBuilderData>>
}

#[derive(Debug, Clone)]
pub struct ConstantBuilderData {
    value: PitchOrValue<f32>,
    text: String
}

impl ConstantBuilderData {
    pub fn new() -> Self {
        let value = PitchOrValue::Value(1.0);
        Self {
            value,
            text: value.to_string()
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        crate::utils::pitch_or_number_input(ui, &mut self.text, &mut self.value);
    }
}

impl ConstantBuilder {
    pub const SPECIFICATION: CircuitSpecification = CircuitSpecification {
        output_names: &["Out"],
        input_names: &[],
        size: egui::vec2(150.0, 100.0),
        playback_size: None,
    };

    const NAME: &'static str = "Constant";

    pub fn new() -> Self {
        Self {
            data: Rc::new(RefCell::new(ConstantBuilderData::new())),
        }
    }

    pub fn data(&self) -> Rc<RefCell<ConstantBuilderData>> {
        self.data.clone()
    }

}

impl CircuitBuilder for ConstantBuilder {
    fn name(&self) -> &str {
        Self::NAME
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        self.data.borrow_mut().show(ui);
    }

    fn specification(&self) -> &'static CircuitSpecification {
        &Self::SPECIFICATION
    }

    fn build(&self, state: &BuildState) -> Box<dyn Circuit> {
        let value = match self.data().borrow().value {
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
