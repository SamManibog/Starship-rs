use std::sync::{atomic::{AtomicBool, Ordering}, Arc};

use crate::circuit::{BuildState, Circuit, CircuitBuilder, CircuitSpecification, CircuitUi};

#[derive(Debug, Clone)]
pub struct SwitchBuilder {
    kind: SwitchKind,
    one_shot_duration: f32,
    one_shot: String
}

impl SwitchBuilder {
    const HOLD_TEXT: &'static str = "Button";
    const TOGGLE_TEXT: &'static str = "Toggle";
    const ONE_SHOT_TEXT: &'static str = "One Shot";

    const SPECIFICATION: CircuitSpecification = CircuitSpecification {
        input_names: &["In"],
        output_names: &["Out"],
        size: egui::vec2(100.0, 100.0),
        playback_size: Some(egui::vec2(100.0, 100.0)),
    };

    pub fn new() -> Self {
        let value = 500.0;
        Self {
            kind: SwitchKind::PressAndHold,
            one_shot_duration: value,
            one_shot: value.to_string()
        }
    }
}

impl CircuitBuilder for SwitchBuilder {
    fn name(&self) -> &str {
        match self.kind {
            SwitchKind::PressAndHold => Self::HOLD_TEXT,
            SwitchKind::Toggle => Self::TOGGLE_TEXT,
            SwitchKind::OneShot => Self::ONE_SHOT_TEXT
        }
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        ui.radio_value(&mut self.kind, SwitchKind::PressAndHold, Self::HOLD_TEXT);
        ui.radio_value(&mut self.kind, SwitchKind::Toggle, Self::TOGGLE_TEXT);
        ui.radio_value(&mut self.kind, SwitchKind::OneShot, Self::ONE_SHOT_TEXT);
        if matches!(self.kind, SwitchKind::OneShot) {
            crate::utils::float_input(
                ui,
                &mut self.one_shot,
                &mut self.one_shot_duration
            );
        }
    }

    fn specification(&self) -> &'static CircuitSpecification {
        &Self::SPECIFICATION
    }

    fn build(&self, state: &BuildState) -> Box<dyn Circuit> {
        match self.kind {
            SwitchKind::PressAndHold => {
                let circuit_state = Arc::new(AtomicBool::new(false));
                state.add_ui(Box::new(ButtonSwitchUi {
                    state: circuit_state.clone(),
                    previous_state: false
                }));
                Box::new(ButtonSwitch {
                    state: circuit_state.clone()
                })
            },
            SwitchKind::Toggle => todo!(),
            SwitchKind::OneShot => todo!()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SwitchKind {
    PressAndHold,
    Toggle,
    OneShot
}

/// Signal passes through when state is true
#[derive(Debug)]
pub struct ButtonSwitch {
    state: Arc<AtomicBool>
}

impl Circuit for ButtonSwitch {
    fn operate(&mut self, inputs: &[f32], outputs: &mut[f32], _: f32) {
        if self.state.load(std::sync::atomic::Ordering::Relaxed) {
            outputs[0] = inputs[0];
        } else {
            outputs[0] = 0.0;
        }
    }
}

#[derive(Debug)]
pub struct ButtonSwitchUi {
    state: Arc<AtomicBool>,
    previous_state: bool
}

impl CircuitUi for ButtonSwitchUi {
    fn show(&mut self, ui: &mut egui::Ui) {
        let new_state = ui.button("on").is_pointer_button_down_on();
        if new_state != self.previous_state {
            self.previous_state = new_state;
            self.state.store(new_state, Ordering::Relaxed);
        }
    }
}

