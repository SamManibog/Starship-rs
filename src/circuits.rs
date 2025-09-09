use crate::circuit::{CircuitBuilder, Circuit, CircuitSpecification};

#[derive(Debug)]
pub struct TestCircuit {
}

impl Circuit for TestCircuit {
    fn operate(&mut self, inputs: &Vec<f32>, outputs: &mut Vec<f32>) {
        let _ = inputs;
        let _ = outputs;
    }
}

#[derive(Debug)]
pub struct TestCircuitBuilder {
}

impl TestCircuitBuilder {
    const SPECIFICATION: CircuitSpecification = CircuitSpecification {
        name: "TestCircuit",
        input_names: &["input1", "input2"],
        output_names: &["output1", "out-outputer"],
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
