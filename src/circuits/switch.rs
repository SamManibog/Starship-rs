use std::sync::{atomic::{AtomicBool, Ordering}, Arc};

use crate::circuit::{BuildState, Circuit, CircuitBuilder, CircuitSpecification, CircuitUi};

#[derive(Debug, Clone)]
pub struct SwitchBuilder {
    kind: SwitchKind,
    one_shot_duration: f32,
    one_shot_text: String,
    declick_duration: f32,
    declick_text: String
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
        let one_shot_value = 500.0;
        let declick_value = 0.0;
        Self {
            kind: SwitchKind::PressAndHold,
            one_shot_duration: one_shot_value,
            one_shot_text: one_shot_value.to_string(),
            declick_duration: declick_value,
            declick_text: declick_value.to_string(),
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
        ui.label("Switch Type:");
        ui.radio_value(&mut self.kind, SwitchKind::PressAndHold, Self::HOLD_TEXT);
        ui.radio_value(&mut self.kind, SwitchKind::Toggle, Self::TOGGLE_TEXT);
        ui.radio_value(&mut self.kind, SwitchKind::OneShot, Self::ONE_SHOT_TEXT);

        ui.separator();
        ui.label("Declick Duration (ms):");
        crate::utils::non_neg_number_input(
            ui,
            &mut self.declick_text,
            &mut self.declick_duration
        );

        if matches!(self.kind, SwitchKind::OneShot) {
            ui.separator();
            ui.label("One Shot Duration (ms):");
            crate::utils::pos_number_input(
                ui,
                &mut self.one_shot_text,
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
                    previous_state: false,
                }));
                if self.declick_duration == 0.0 {
                    Box::new(SwitchNoDeclick {
                        state: circuit_state.clone(),
                    })
                } else {
                    Box::new(SwitchDeclick {
                        state: circuit_state.clone(),
                        declick_index: 0.0,
                        max_declick_index: self.declick_duration / 1000.0
                    })
                }
            },
            SwitchKind::Toggle => {
                let circuit_state = Arc::new(AtomicBool::new(false));
                state.add_ui(Box::new(ToggleSwitchUi {
                    state: circuit_state.clone(),
                    current_state: false,
                }));
                if self.declick_duration == 0.0 {
                    Box::new(SwitchNoDeclick {
                        state: circuit_state.clone(),
                    })
                } else {
                    Box::new(SwitchDeclick {
                        state: circuit_state.clone(),
                        declick_index: 0.0,
                        max_declick_index: self.declick_duration / 1000.0
                    })
                }
            },
            SwitchKind::OneShot => {
                let circuit_state = Arc::new(AtomicBool::new(false));
                state.add_ui(Box::new(ButtonSwitchUi {
                    state: circuit_state.clone(),
                    previous_state: false,
                }));
                if self.declick_duration == 0.0 {
                    Box::new(OneShotNoDeclick {
                        state: circuit_state.clone(),
                        one_shot_index: 0.0,
                        max_one_shot_index: self.one_shot_duration / 1000.0,
                        handle: true,
                    })
                } else {
                    Box::new(OneShotDeclick {
                        state: circuit_state.clone(),
                        one_shot_index: 0.0,
                        max_one_shot_index: self.one_shot_duration / 1000.0,
                        declick_index: 0.0,
                        max_declick_index: self.declick_duration / 1000.0,
                        handle: true,
                    })
                }
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SwitchKind {
    PressAndHold,
    Toggle,
    OneShot
}

/// Signal passes through when state is true. No declicking.
#[derive(Debug)]
pub struct SwitchNoDeclick {
    state: Arc<AtomicBool>,
}

impl Circuit for SwitchNoDeclick {
    fn operate(&mut self, inputs: &[f32], outputs: &mut[f32], _: f32) {
        if self.state.load(Ordering::Relaxed) {
            outputs[0] = inputs[0];
        } else {
            outputs[0] = 0.0;
        };
    }
}

/// Signal passes through when state is true. Has declicking.
#[derive(Debug)]
pub struct SwitchDeclick {
    state: Arc<AtomicBool>,
    declick_index: f32,
    max_declick_index: f32
}

impl Circuit for SwitchDeclick {
    fn operate(&mut self, inputs: &[f32], outputs: &mut[f32], delta: f32) {
        let declick_delta = if self.state.load(Ordering::Relaxed) {
            delta
        } else {
            -delta
        };
        self.declick_index = (self.declick_index + declick_delta)
                .clamp(0.0, self.max_declick_index);
        outputs[0] += inputs[0] * (self.declick_index / self.max_declick_index);
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

#[derive(Debug)]
pub struct ToggleSwitchUi {
    state: Arc<AtomicBool>,
    current_state: bool
}

impl CircuitUi for ToggleSwitchUi {
    fn show(&mut self, ui: &mut egui::Ui) {
        let text = if self.current_state {
            "on"
        } else {
            "off"
        };
        if ui.button(text).clicked() {
            self.current_state = !self.current_state;
            self.state.store(self.current_state, Ordering::Relaxed);
        }
    }
}

/// Signal passes through when state is true. No declicking.
#[derive(Debug)]
pub struct OneShotNoDeclick {
    state: Arc<AtomicBool>,
    one_shot_index: f32,
    max_one_shot_index: f32,

    // true if ready to handle state == true
    handle: bool,
}

impl Circuit for OneShotNoDeclick {
    fn operate(&mut self, inputs: &[f32], outputs: &mut[f32], delta: f32) {
        let pressed = self.state.load(Ordering::Relaxed);
        if pressed && self.handle {
            self.one_shot_index = self.max_one_shot_index;
            self.handle = false;
        } else {
            self.one_shot_index -= delta;
            if self.one_shot_index <= 0.0 {
                self.one_shot_index = 0.0;

                if !pressed {
                    self.handle = true;
                }
            }
        }
        if self.one_shot_index > 0.0 {
            outputs[0] = inputs[0];
        } else {
            outputs[0] = 0.0;
        }
    }
}

/// Signal passes through when state is true. Has declicking.
#[derive(Debug)]
pub struct OneShotDeclick {
    state: Arc<AtomicBool>,
    one_shot_index: f32,
    max_one_shot_index: f32,
    declick_index: f32,
    max_declick_index: f32,

    // true if ready to handle state == true
    handle: bool,
}

impl Circuit for OneShotDeclick {
    fn operate(&mut self, inputs: &[f32], outputs: &mut[f32], delta: f32) {
        let pressed = self.state.load(Ordering::Relaxed);
        if pressed && self.handle {
            self.one_shot_index = self.max_one_shot_index;
            self.handle = false;
        } else {
            self.one_shot_index -= delta;
            if self.one_shot_index <= 0.0 {
                self.one_shot_index = 0.0;

                if !pressed {
                    self.handle = true;
                }
            }
        }
        let declick_delta = if self.one_shot_index > 0.0 {
            delta
        } else {
            -delta
        };
        self.declick_index = (self.declick_index + declick_delta)
                .clamp(0.0, self.max_declick_index);
        outputs[0] += inputs[0] * (self.declick_index / self.max_declick_index);
    }
}

