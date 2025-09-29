use crate::circuit::{BuildState, Circuit, CircuitBuilder, CircuitSpecification};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OscillatorKind {
    Sine,
    Saw,
    Square,
    Triangle,
}

impl OscillatorKind {
    const SINE_TEXT: &'static str = "Sine Wave";
    const SAW_TEXT: &'static str = "Sawtooth Wave";
    const SQR_TEXT: &'static str = "Square Wave";
    const TRI_TEXT: &'static str = "Triangle Wave";

    fn display_string(&self) -> &'static str {
        match self {
            Self::Sine => Self::SINE_TEXT,
            Self::Saw => Self::SAW_TEXT,
            Self::Square => Self::SQR_TEXT,
            Self::Triangle => Self::TRI_TEXT,
        }
    }
}

impl std::fmt::Display for OscillatorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.display_string()
        )
    }
}

#[derive(Debug, Clone)]
pub struct OscillatorBuilder {
    kind: OscillatorKind
}

impl OscillatorBuilder {
    const SPECIFICATION: CircuitSpecification = CircuitSpecification {
        input_names: &["Amplitude", "Frequency"],
        output_names: &["Out"],
        size: egui::vec2(200.0, 200.0),
        playback_size: None,
    };

    pub fn new() -> Self {
        Self{
            kind: OscillatorKind::Sine
        }
    }
}

impl CircuitBuilder for OscillatorBuilder {
    fn show(&mut self, ui: &mut egui::Ui) {
        ui.radio_value(&mut self.kind, OscillatorKind::Sine, OscillatorKind::SINE_TEXT);
        ui.radio_value(&mut self.kind, OscillatorKind::Triangle, OscillatorKind::TRI_TEXT);
        ui.radio_value(&mut self.kind, OscillatorKind::Saw, OscillatorKind::SAW_TEXT);
        ui.radio_value(&mut self.kind, OscillatorKind::Square, OscillatorKind::SQR_TEXT);
    }

    fn name(&self) -> &str {
        self.kind.display_string()
    }

    fn specification(&self) -> &'static CircuitSpecification {
        &Self::SPECIFICATION
    }

    fn build(&self, _: &BuildState) -> Box<dyn Circuit> {
        match self.kind {
            OscillatorKind::Sine => Box::new(Sine::default()),
            OscillatorKind::Saw => Box::new(Saw::default()),
            OscillatorKind::Square => Box::new(Square::default()),
            OscillatorKind::Triangle => Box::new(Triangle::default()),
        }
    }
}

#[derive(Debug, Default)]
pub struct Sine {
    index: f32
}

impl Circuit for Sine {
    fn operate(&mut self, inputs: &[f32], outputs: &mut[f32], delta: f32) {
        //Sine function with amplitude inputs[0] and frequency 1hz
        outputs[0] = inputs[0] * f32::sin(self.index * std::f32::consts::TAU);

        //Incriment index by interval * frequency, effectively making sine function
        //have a frequency of inputs[1]
        self.index += delta * inputs[1];
        self.index %= 1.0;
    }
}

#[derive(Debug, Default)]
pub struct Saw {
    index: f32
}

impl Circuit for Saw {
    fn operate(&mut self, inputs: &[f32], outputs: &mut[f32], delta: f32) {
        //reverse sawtooth function with amplitude inputs[0] and frequency 1hz
        outputs[0] = inputs[0] * (self.index - 1.0);

        //Incriment index by interval * frequency, effectively making sine function
        //have a frequency of inputs[1]
        self.index += delta * inputs[1] * 2.0;
        self.index %= 2.0;
    }
}

#[derive(Debug, Default)]
pub struct Square {
    index: f32
}

impl Circuit for Square {
    fn operate(&mut self, inputs: &[f32], outputs: &mut[f32], delta: f32) {
        //squarewave function with amplitude inputs[0] and frequency 1hz
        outputs[0] = inputs[0] * (2.0 * f32::floor(2.0 * (self.index % 2.0)) - 1.0);

        //Incriment index by interval * frequency, effectively making sine function
        //have a frequency of inputs[1]
        self.index += delta * inputs[1];
        self.index %= 1.0;
    }
}

#[derive(Debug)]
pub struct Triangle {
    index: f32
}

impl Default for Triangle {
    fn default() -> Self {
        Self {
            index: 0.75
        }
    }
}

impl Circuit for Triangle {
    fn operate(&mut self, inputs: &[f32], outputs: &mut[f32], delta: f32) {
        //triangle function with amplitude inputs[0] and frequency 1hz
        outputs[0] = inputs[0] * ( f32::abs(4.0 * ( self.index % 1.0 ) - 2.0) - 1.0 );

        //Incriment index by interval * frequency, effectively making sine function
        //have a frequency of inputs[1]
        self.index += delta * inputs[1];
        self.index %= 1.0;
    }
}

