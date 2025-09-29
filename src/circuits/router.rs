use crate::circuit::{BuildState, Circuit, CircuitBuilder, CircuitSpecification};

#[derive(Debug, Clone)]
pub struct RouterBuilder {}

impl RouterBuilder {
    const SPECIFICATION: CircuitSpecification = CircuitSpecification {
        input_names: &["In"],
        output_names: &["Out"],
        size: egui::vec2(100.0, 70.0),
        playback_size: None,
    };

    const NAME: &'static str = "Router";

    pub fn new() -> Self {
        Self{}
    }
}

impl CircuitBuilder for RouterBuilder {
    fn name(&self) -> &str {
        Self::NAME
    }

    fn specification(&self) -> &'static CircuitSpecification {
        &Self::SPECIFICATION
    }

    fn build(&self, _: &BuildState) -> Box<dyn Circuit> {
        Box::new(Router::default())
    }
}

#[derive(Debug, Default)]
pub struct Router {}

impl Circuit for Router {
    fn operate(&mut self, inputs: &[f32], outputs: &mut[f32], _: f32) {
        outputs[0] = inputs[0];
    }
}
