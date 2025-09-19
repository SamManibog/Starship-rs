use std::{collections::{HashMap, HashSet}, sync::Arc};

use eframe;
use egui::{epaint::CubicBezierShape, scroll_area::ScrollBarVisibility, Color32};

use crate::{
    circuit::{CircuitBuilderFrontend, PortUi}, circuit_id::{CircuitId, CircuitPortId, ConnectionId, PortKind}, circuits::TestCircuitBuilder, connection_manager::ConnectionManager, connection_proposal::{ConnectionProposal, ConnectionProposalState}
};

#[derive(Debug)]
enum Drag {
    SceneDrag(egui::Vec2),
    ModuleDrag(usize, egui::Vec2),
    NoDrag
}

impl Default for Drag {
    fn default() -> Self {
        Self::NoDrag
    }
}

#[derive(Debug)]
pub struct StarshipApp {
    cam_pos: egui::Vec2,
    circuit_builders: Vec<CircuitBuilderFrontend>,
    area_positions: Vec<egui::Pos2>,
    connections: ConnectionManager,
    connection_proposal: ConnectionProposal,
}

impl Default for StarshipApp {
    fn default() -> Self {
        let mut circuit_builders = vec![];
        for _ in 1..=2 {
            circuit_builders.push(
                CircuitBuilderFrontend::new(
                    unsafe { CircuitId::new() },
                    Box::new(TestCircuitBuilder::new())
                )
            );
        }
        Self {
            cam_pos: egui::vec2(0.0, 0.0),
            circuit_builders,
            area_positions: vec![egui::pos2(400.0, 400.0), egui::pos2(200.0, 200.0)],
            connections: Default::default(),
            connection_proposal: Default::default(),
        }
    }
}

impl StarshipApp {
    pub const CONNECT_COLORS: &[egui::Color32] = &[
        egui::Color32::RED,
        egui::Color32::BLUE,
        egui::Color32::GREEN,
        egui::Color32::MAGENTA,
        egui::Color32::YELLOW,
    ];

    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        cc.egui_ctx.set_style({
            let mut style = egui::Style::default();
            style.wrap_mode = Some(egui::TextWrapMode::Extend);
            style.interaction.selectable_labels = false;
            Arc::new(style)
        });
        Default::default()
    }

}

impl eframe::App for StarshipApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                if ui.button("Quit").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                ui.add_space(16.0);

                egui::warn_if_debug_build(ui);
            });
        });


        egui::SidePanel::right("right_panel")
            .max_width(400.0)
            .show(ctx, |ui| {
            ui.separator();
        });

        //A map CircuitPortId -> egui::Pos2
        //used to draw connections between ports
        let mut port_positions = HashMap::<CircuitPortId, egui::Pos2>::new();

        let drag = egui::CentralPanel::default()
            .show(ctx, |ui| {
                //check if scene is dragged
                let scene_response = ui.interact(
                    ui.max_rect(),
                    egui::Id::new("Scene"),
                    egui::Sense::DRAG
                );

                let mut mod_response: Option<(usize, egui::Response)> = None;
                for (index, builder) in self.circuit_builders.iter_mut().enumerate() {
                    let response = builder.show(
                        self.area_positions[index] - self.cam_pos,
                        ui,
                        &mut port_positions,
                        &mut self.connection_proposal,
                    );
                    if response.dragged() {
                        mod_response = Some((index, response))
                    }
                }

                //Draw Connections and handle connection state
                {
                    let painter = ui.painter();

                    self.connections.draw_connections(painter, &port_positions);

                    //draw new connections and handle new connection state
                    if let ConnectionProposalState::Started(connection) = &self.connection_proposal.state() {
                        //ensure we are still dragging and on-screen
                        let mouse_pos_opt = ui.ctx().input(|input| {
                            if input.pointer.primary_released() {
                                None
                            } else {
                                input.pointer.latest_pos()
                            }
                        });

                        //if mouse state is good, draw the connection
                        //otherwise, cancel the connection
                        if let Some(mouse_pos) = mouse_pos_opt {
                            let (start, end) = if connection.port_id.kind() == PortKind::Input {
                                (mouse_pos, port_positions[&connection])
                            } else {
                                (port_positions[&connection], mouse_pos)
                            };
                            //Self::draw_connection(painter, start, end);
                            ConnectionManager::draw_connection(painter, Color32::WHITE, start, end);
                        } else {
                            self.connection_proposal.cancel();
                        }

                    } else if let ConnectionProposalState::Finalized(start, end) = *self.connection_proposal.state() {
                        self.connections.add_connection(ConnectionId::new(start, end));
                        self.connection_proposal.cancel();
                    }
                }

                if let Some((index, response)) = mod_response {
                    Drag::ModuleDrag(index, response.drag_delta())
                } else if scene_response.dragged() {
                    Drag::SceneDrag(scene_response.drag_delta())
                } else {
                    Drag::NoDrag
                }
            }).inner;

        match drag {
            Drag::ModuleDrag(index, delta) => { self.area_positions[index] += delta; },
            Drag::SceneDrag(delta) => { self.cam_pos -= delta; },
            Drag::NoDrag => {},
        }

    }
}
