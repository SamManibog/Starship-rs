use crate::circuit::{CircuitBuilder, Circuit, CircuitSpecification};

mod sine;
pub use sine::*;

#[derive(Debug)]
pub struct TestCircuit {
}

impl Circuit for TestCircuit {
    fn operate(&mut self, inputs: &[f32], outputs: &mut[f32]) {
        let _ = inputs;
        let _ = outputs;
        todo!()
    }
}

#[derive(Debug)]
pub struct TestCircuitBuilder {
}

impl TestCircuitBuilder {
    const SPECIFICATION: CircuitSpecification = CircuitSpecification {
        name: "TestCircuit",
        input_names: &["input1", "input2"],
        output_names: &["output1", "outputer"],
    };

    pub fn new() -> Self {
        Self {}
    }
}

impl CircuitBuilder for TestCircuitBuilder {
    fn build(&self) -> Box<dyn Circuit> {
        Box::new(TestCircuit{})
    }

    fn specification(&self) -> &'static CircuitSpecification {
        &Self::SPECIFICATION
    }

    fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.button("test button")
    }
}
