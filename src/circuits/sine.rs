use crate::circuit::{CircuitBuilder, Circuit, ConnectionSpecification};

#[derive(Debug, Clone)]
pub struct SineBuilder {
}

impl SineBuilder {
    const SPECIFICATION: ConnectionSpecification = ConnectionSpecification {
        name: "Sinewave",
        input_names: &["Amplitude", "Frequency"],
        output_names: &["Out"],
        size: egui::vec2(200.0, 200.0),
    };

    pub fn new() -> Self {
        Self{}
    }
}

impl CircuitBuilder for SineBuilder {
    fn specification(&self) -> &'static ConnectionSpecification {
        &Self::SPECIFICATION
    }

    fn build(&self) -> Box<dyn Circuit> {
        Box::new(Sine::default())
    }
}

#[derive(Debug, Default)]
pub struct Sine {
    index: f32
}

impl Circuit for Sine {
    fn operate(&mut self, inputs: &[f32], outputs: &mut[f32]) {
        //Sine function with amplitude inputs[0] and frequency 1hz
        outputs[0] = inputs[0] * f32::sin(self.index * std::f32::consts::TAU);

        //Incriment index by interval * frequency, effectively making sine function
        //have a frequency of inputs[1]
        self.index += (crate::constants::SAMPLE_INTERVAL as f32) * inputs[1];
        self.index %= 1.0;
    }
}
