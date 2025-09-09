use crate::circuit_id::CircuitId;

///Designator for an input or output port
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortKind {
    Input,
    Output
}

///The identifier of a port
///has two components: index and kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortId {
    data: i32,
}

impl PortId {
    pub fn new(index: usize, kind: PortKind) -> Self {
        let sign: i32 = if kind == PortKind::Input { 1 } else { -1 };
        let magnitude: i32 = index as i32 + 1;
        Self {
            data: sign * magnitude
        }
    }

    pub fn kind(&self) -> PortKind {
        if self.data > 0 {
            PortKind::Input
        } else {
            PortKind::Output
        }
    }

    pub fn index(&self) -> usize {
        (self.data.abs() - 1) as usize
    }
}

///The identifier for a port on a specific circuit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CircuitPortId {
    pub circuit_id: CircuitId,
    pub port_id: PortId,
}

impl CircuitPortId {
    pub fn new(circuit_id: CircuitId, port_id: PortId) -> Self {
        Self {
            circuit_id,
            port_id
        }
    }

    pub fn circuit_id(&self) -> CircuitId {
        self.circuit_id
    }

    pub fn port_id(&self) -> PortId {
        self.port_id
    }
}

pub struct CircuitSpecification {
    pub name: &'static str,
    pub input_names: &'static[&'static str],
    pub output_names: &'static[&'static str]
}

///Creates a circuit based on user parameters
pub trait CircuitBuilder: std::fmt::Debug {
    ///Draw the circuit UI to the screen. Passed to egui's show function.
    ///Do not attempt to handle circuit connections in this step.
    fn show(&mut self, ui: &mut egui::Ui) -> egui::Response;

    ///gets the specification for the circuit
    fn specification(&self) -> &'static CircuitSpecification;

    ///Build the associated circuit
    fn build(&self) -> Box<dyn Circuit>;

    ///Called when adding an input target to a circuit
    fn on_input_added(&mut self, port: PortId) { let _ = port; }

    ///Called when removing an input target to a circuit
    fn on_input_removed(&mut self, port: PortId) { let _ = port; }
}

///A circuit that processes signals into outputs
pub trait Circuit: std::fmt::Debug {
    ///Handles a vector of signals to produce some output signals.
    fn operate(&mut self, inputs: &Vec<f32>, outputs: &mut Vec<f32>);
}

///Handles the ui used to build a circuit
#[derive(Debug)]
pub struct CircuitBuilderFrontend {
    id: CircuitId,
    builder: Box<dyn CircuitBuilder>,
    inputs: Vec<Vec<CircuitPortId>>,
}

impl CircuitBuilderFrontend {
    const DEFAULT_DIMENSIONS: egui::Vec2 = egui::vec2(150.0, 150.0);

    ///Creates a new instance
    pub fn new(id: CircuitId, builder: Box<dyn CircuitBuilder>) -> Self {
        let mut inputs = vec![];
        for _ in 0..builder.specification().input_names.len() {
            inputs.push(vec![])
        }
        Self {
            id,
            builder,
            inputs,
        }
    }

    ///Gets the associated builder
    pub fn builder(&self) -> &Box<dyn CircuitBuilder> {
        &self.builder
    }

    ///Gets the list of inputs
    pub fn inputs(&self) -> &Vec<Vec<CircuitPortId>> {
        &self.inputs
    }

    ///Adds an input source to the list of inputs at the given port
    pub fn add_input(&mut self, port: PortId, source: CircuitPortId) {
        //ensure port is an input port
        assert!(port.kind() == PortKind::Input);

        //ensure source is an output port
        assert!(source.port_id().kind() == PortKind::Output);

        self.inputs[port.index()].push(source);
    }

    pub fn show(&mut self, position: egui::Pos2, ui: &mut egui::Ui) -> egui::Response {
        let ui_builder = egui::UiBuilder::new()
            .max_rect(egui::Rect::from_min_size(position, Self::DEFAULT_DIMENSIONS));

        ui.scope_builder(ui_builder, |ui| {
            //detect dragging on title (used for moving the whole circuit)
            let response = egui::Frame::new()
                .fill(ui.ctx().style().visuals.faint_bg_color)
                .stroke(ui.ctx().style().visuals.window_stroke)
                .show(ui, |ui| {
                    ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                        ui.add(
                            egui::Label::new(self.builder.specification().name)
                                .sense(egui::Sense::DRAG)
                        )
                    }).inner
                }).inner;

            //draw IO
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    for input in self.builder.specification().input_names {
                        ui.label(*input);
                    }
                });
                ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |_| {});
                ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
                    for output in self.builder.specification().output_names {
                        ui.label(*output);
                    }
                });
            });

            //draw builder
            self.builder.show(ui);

            response
        }).inner
    }
}

impl std::hash::Hash for CircuitBuilderFrontend {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}
