use std::collections::HashMap;

use egui::Color32;

use crate::{
    circuit_id::{CircuitId, CircuitPortId, PortId, PortKind},
    circuit_input::CircuitInput,
    circuit::CircuitSpecification
};

/// Handles the ui used to build a circuit
#[derive(Debug)]
pub struct ConnectionBuilder {
    id: CircuitId,
    specification: &'static CircuitSpecification,
}

impl ConnectionBuilder {
    /// Creates a new instance
    pub fn new(id: CircuitId, specification: &'static CircuitSpecification) -> Self {
        Self {
            id,
            specification,
        }
    }

    /// Gets the id of the circuit
    pub fn id(&self) -> CircuitId {
        self.id
    }

    /// Gets the associated specification
    pub fn specification(&self) -> &'static CircuitSpecification {
        self.specification
    }

    pub fn show(
        &mut self,
        position: egui::Pos2,
        ui: &mut egui::Ui,
        register: &mut HashMap<CircuitPortId, egui::Pos2>,
        input: &mut CircuitInput,
        highlight: bool,
        name: &str
    ) -> egui::Response {
        let ui_builder = egui::UiBuilder::new()
            .sense(egui::Sense::all())
            .max_rect(egui::Rect::from_min_size(
                position,
                self.specification.size
            ));

        ui.scope_builder(ui_builder, |ui| {
            let mut stroke = ui.ctx().style().visuals.window_stroke;
            if highlight {
                stroke.color = Color32::WHITE;
            }
            egui::Frame::new()
                .fill(ui.ctx().style().visuals.window_fill)
                .stroke(stroke)
                .inner_margin(4.0)
                .corner_radius(12)
                .show(ui, |ui| {
                    ui.vertical_centered_justified(|ui| {
                        ui.label(name);
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            self.draw_ports(
                                ui,
                                register,
                                input,
                                self.specification.input_names,
                                PortKind::Input
                            );
                        });
                        ui.with_layout(
                            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                            |_| { }
                        );
                        ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
                            self.draw_ports(
                                ui,
                                register,
                                input,
                                self.specification.output_names,
                                PortKind::Output
                            );
                        });
                    });
                });

            ui.response()
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

impl std::hash::Hash for ConnectionBuilder {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

#[derive(Debug)]
pub struct PortUi<'a> {
    /// The id of the associated port
    id: CircuitPortId,

    /// A mutable reference to the app state's new_connection member, 
    /// which is used to handle the possible creation of a new connection
    connection_proposal: &'a mut CircuitInput
}

impl<'a> PortUi<'a> {
    /// Radius of the port when disconnected
    pub const UNFILLED_RADIUS: f32 = 5.0;

    /// Color of the port when disconnected
    pub const UNFILLED_COLOR: egui::Color32 = egui::Color32::BLACK;

    /// Radius of the port when connected
    pub const FILLED_RADIUS: f32 = 6.0;

    /// Color of the port when connected
    pub const FILLED_COLOR: egui::Color32 = egui::Color32::BLACK;

    /// Color of the port when hovered
    pub const HOVERED_COLOR: egui::Color32 = egui::Color32::WHITE;

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
        if response.hovered() {
            painter.circle_filled(center, Self::FILLED_RADIUS, Self::HOVERED_COLOR);
        }
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
