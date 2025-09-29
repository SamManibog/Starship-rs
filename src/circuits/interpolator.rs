use crate::circuit::{BuildState, Circuit, CircuitBuilder, CircuitSpecification};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InterpolatorKind {
    Linear,
    LogLinear,
}

impl InterpolatorKind {
    const LINEAR_TEXT: &'static str = "Linear Interpolator";
    const LOG_LINEAR_TEXT: &'static str = "Log-Linear Interpolator";

    fn display_string(&self) -> &'static str {
        match self {
            Self::Linear => Self::LINEAR_TEXT,
            Self::LogLinear => Self::LOG_LINEAR_TEXT,
        }
    }
}

impl std::fmt::Display for InterpolatorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.display_string()
        )
    }
}

#[derive(Debug, Clone)]
pub struct InterpolatorBuilder {
    kind: InterpolatorKind
}

impl InterpolatorBuilder {
    const SPECIFICATION: CircuitSpecification = CircuitSpecification {
        input_names: &["End", "Start", "Completion"],
        output_names: &["Out"],
        size: egui::vec2(200.0, 200.0),
        playback_size: None,
    };

    pub fn new() -> Self {
        Self{
            kind: InterpolatorKind::Linear
        }
    }
}

impl CircuitBuilder for InterpolatorBuilder {
    fn show(&mut self, ui: &mut egui::Ui) {
        ui.label("Interpolation Type:");
        ui.radio_value(&mut self.kind, InterpolatorKind::Linear, InterpolatorKind::LINEAR_TEXT);
        ui.radio_value(&mut self.kind, InterpolatorKind::LogLinear, InterpolatorKind::LOG_LINEAR_TEXT);
    }

    fn name(&self) -> &str {
        self.kind.display_string()
    }

    fn specification(&self) -> &'static CircuitSpecification {
        &Self::SPECIFICATION
    }

    fn build(&self, _: &BuildState) -> Box<dyn Circuit> {
        match self.kind {
            InterpolatorKind::Linear => Box::new(Lerper{}),
            InterpolatorKind::LogLinear => Box::new(LogLerper{}),
        }
    }
}

#[derive(Debug, Default)]
pub struct Lerper {}

impl Circuit for Lerper {
    fn operate(&mut self, inputs: &[f32], outputs: &mut[f32], _: f32) {
        let end = inputs[0];
        let start = inputs[1];
        let completion = inputs[2].clamp(0.0, 1.0);
        outputs[0] = (end - start) * completion + start;
    }
}

#[derive(Debug, Default)]
pub struct LogLerper {}

impl Circuit for LogLerper {
    fn operate(&mut self, inputs: &[f32], outputs: &mut[f32], _: f32) {
        let end = f32::log2(inputs[0].abs() + 1.0);
        let start = f32::log2(inputs[1].abs() + 1.0);
        let completion = inputs[2].clamp(0.0, 1.0);
        outputs[0] = f32::powf(2.0, (end - start) * completion + start) - 1.0;
    }
}
