use crate::circuit::{CircuitBuilder, Circuit, ConnectionSpecification};

#[derive(Debug, Clone)]
pub struct RouterBuilder {}

impl RouterBuilder {
    const SPECIFICATION: ConnectionSpecification = ConnectionSpecification {
        input_names: &["In"],
        output_names: &["Out"],
        size: egui::vec2(100.0, 70.0),
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

    fn specification(&self) -> &'static ConnectionSpecification {
        &Self::SPECIFICATION
    }

    fn build(&self) -> Box<dyn Circuit> {
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
