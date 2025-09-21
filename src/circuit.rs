use std::collections::HashMap;

use crate::{circuit_input::CircuitInput, circuit_id::{CircuitId, CircuitPortId, PortId, PortKind}};

pub struct CircuitSpecification {
    pub name: &'static str,
    pub input_names: &'static[&'static str],
    pub output_names: &'static[&'static str]
}

pub struct CircuitBuilderSpecification {
    pub display_name: String,
    pub instance: Box<dyn Fn()->Box<dyn CircuitBuilder>>
}

impl CircuitBuilderSpecification {
    pub fn new(name: &str, instance: impl Fn()->Box<dyn CircuitBuilder> + 'static) -> Self {
        Self {
            display_name: name.into(),
            instance: Box::new(instance)
        }
    }
}

impl std::fmt::Debug for CircuitBuilderSpecification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "display_name: {}", self.display_name)
    }
}

///Creates a circuit based on user parameters
pub trait CircuitBuilder: std::fmt::Debug {
    ///Draw the circuit UI to the screen. Passed to egui's show function.
    ///Do not attempt to handle circuit connections in this step.
    fn show(&mut self, ui: &mut egui::Ui) { let _ = ui; }

    ///gets the specification for the circuit
    fn specification(&self) -> &'static CircuitSpecification;

    ///Build the associated circuit
    fn build(&self) -> Box<dyn Circuit>;

    ///Called when adding an input target to a circuit
    fn on_input_added(&mut self, port: PortId) { let _ = port; }

    ///Called when removing an input target to a circuit
    fn on_input_removed(&mut self, port: PortId) { let _ = port; }

    ///Request a size for the entire UI.
    ///This size will be filled with the title, IO ports, padding, etc. along with your custom UI.
    ///Called every frame before drawing.
    fn request_size(&self) -> Option<egui::Vec2> {
        return None;
    }
}

///A circuit that processes signals into outputs
pub trait Circuit: std::fmt::Debug {
    ///Handles a vector of signals to produce some output signals.
    fn operate(&mut self, inputs: &[f32], outputs: &mut[f32]);
}

///Handles the ui used to build a circuit
#[derive(Debug)]
pub struct CircuitBuilderFrontend {
    id: CircuitId,
    builder: Box<dyn CircuitBuilder>,
}

impl CircuitBuilderFrontend {
    pub const MINIMUM_WIDTH: f32 = 200.0;
    pub const DEFAULT_DIMENSIONS: egui::Vec2 = egui::vec2(Self::MINIMUM_WIDTH, 200.0);

    ///Creates a new instance
    pub fn new(id: CircuitId, builder: Box<dyn CircuitBuilder>) -> Self {
        Self {
            id,
            builder,
        }
    }

    ///Gets the id of the circuit
    pub fn id(&self) -> CircuitId {
        self.id
    }

    ///Gets the associated builder
    pub fn builder(&self) -> &Box<dyn CircuitBuilder> {
        &self.builder
    }

    pub fn show(
        &mut self,
        position: egui::Pos2,
        ui: &mut egui::Ui,
        register: &mut HashMap<CircuitPortId, egui::Pos2>,
        input: &mut CircuitInput
    ) -> egui::Response {
        let ui_builder = {
            let mut dimensions = self.builder.request_size().unwrap_or(Self::DEFAULT_DIMENSIONS);
            dimensions.x = Self::MINIMUM_WIDTH.max(dimensions.x);
            egui::UiBuilder::new()
                .sense(egui::Sense::all())
                .max_rect(egui::Rect::from_min_size(
                    position,
                    dimensions
                ))
        };

        ui.scope_builder(ui_builder, |ui| {
            //detect dragging on title (used for moving the whole circuit)
            let response = ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                egui::Frame::new()
                    .fill(ui.ctx().style().visuals.window_fill)
                    .stroke(ui.ctx().style().visuals.window_stroke)
                    .inner_margin(4.0)
                    .corner_radius(12.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::Label::new(self.builder.specification().name)
                                .sense(egui::Sense::click_and_drag())
                        )
                    }).inner
            }).inner;

            //draw IO
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    self.draw_ports(
                        ui,
                        register,
                        input,
                        self.builder.specification().input_names,
                        PortKind::Input
                    );
                });
                ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |_| { });
                ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
                    self.draw_ports(
                        ui,
                        register,
                        input,
                        self.builder.specification().output_names,
                        PortKind::Output
                    );
                });
            });

            //draw builder
            self.builder.show(ui);

            if response.clicked() {
                input.circuit_click(self.id);
            }

            response
        }).inner
    }

    fn draw_ports(
        &self,
        ui: &mut egui::Ui,
        register: &mut HashMap<CircuitPortId, egui::Pos2>,
        connection: &mut CircuitInput,
        names: &[&str],
        kind: PortKind
    ) {
        for (idx, name) in names.iter().enumerate() {
            ui.horizontal(|ui| {
                let id = CircuitPortId::new(
                        self.id,
                        PortId::new(idx, kind)
                    );
                register.insert(
                    id,
                    ui.add(PortUi::new(id, connection)).rect.center()
                );
                ui.label(*name);
            });
        }
    }

}

impl std::hash::Hash for CircuitBuilderFrontend {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

#[derive(Debug)]
pub struct PortUi<'a> {
    ///The id of the associated port
    id: CircuitPortId,

    ///A mutable reference to the app state's new_connection member, 
    ///which is used to handle the possible creation of a new connection
    connection_proposal: &'a mut CircuitInput
}

impl<'a> PortUi<'a> {
    ///Radius of the port when disconnected
    pub const UNFILLED_RADIUS: f32 = 5.0;

    ///Color of the port when disconnected
    pub const UNFILLED_COLOR: egui::Color32 = egui::Color32::BLACK;

    ///Radius of the port when connected
    pub const FILLED_RADIUS: f32 = 6.0;

    ///Color of the port when connected
    pub const FILLED_COLOR: egui::Color32 = egui::Color32::BLACK;

    pub fn new(id: CircuitPortId, connection: &'a mut CircuitInput) -> Self {
        Self {
            id,
            connection_proposal: connection
        }
    }
}

impl egui::Widget for PortUi<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (response, painter) = ui.allocate_painter(
            egui::vec2(PortUi::FILLED_RADIUS * 2.0, PortUi::FILLED_RADIUS * 2.0),
            egui::Sense::click_and_drag()
        );
        let center = response.rect.center();
        painter.circle_filled(center, Self::UNFILLED_RADIUS, Self::UNFILLED_COLOR);
        if response.drag_started() {
            response.dnd_set_drag_payload::<CircuitPortId>(self.id);
            let _ = self.connection_proposal.start(self.id);
        } else if let Some(_) = response.dnd_release_payload::<CircuitPortId>() {
            let _ = self.connection_proposal.propose(self.id);
            let _ = self.connection_proposal.finalize();
        } else if response.clicked() {
            self.connection_proposal.click(self.id);
        }
        response
    }
}
