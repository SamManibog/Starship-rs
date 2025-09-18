use std::{collections::{HashMap, HashSet}, sync::Arc};

use eframe;
use egui::epaint::CubicBezierShape;

use crate::{
    circuit::{CircuitBuilderFrontend, PortUi},
    circuit_id::{CircuitId, CircuitPortId, ConnectionId, PortKind},
    circuits::TestCircuitBuilder,
    connection_proposal::{ConnectionProposal, ConnectionProposalState},
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
    connections: Vec<ConnectionId>,
    connection_set: HashSet<ConnectionId>,
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
            connection_set: Default::default(),
            connection_proposal: Default::default(),
        }
    }
}

impl StarshipApp {
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

    ///Adds the given connection to the list of connections.
    ///Returns true if the connection was successfully added.
    fn add_connection(&mut self, connection: ConnectionId) -> bool {
        if !self.connection_set.contains(&connection) {
            self.connections.push(connection);
            self.connection_set.insert(connection);
            true
        } else {
            false
        }
    }

    ///Removes the given connection from the list of connections.
    ///Returns true if the connection was successfully removed
    fn remove_connection(&mut self, connection: ConnectionId) -> bool {
        if self.connection_set.contains(&connection) {
            self.connections.retain(|entry| {
                *entry != connection 
            });
            self.connection_set.remove(&connection);
            true
        } else {
            false
        }
    }

    const CONNECT_Y_FACTOR: f32 = 1000.0;
    const CONNECT_POSITIVE_X_FACTOR: f32 = 1.0 / 1.5;
    const CONNECT_NEGATIVE_X_FACTOR: f32 = 0.5;
    const CONNECT_MIN_X: f32 = 100.0;
    const CONNECT_MAX_X: f32 = 200.0;
    const CONNECT_THICKNESS: f32 = 1.0;
    ///gets the points for the cubic bezier connecting the start and end points
    fn get_connection_points(start: egui::Pos2, end: egui::Pos2) -> [egui::Pos2; 4] {
        let mut diff_x = (end.x - start.x).abs();
        let mut diff_y = 0.0;
        if start.x > end.x {
            diff_y = (end.y - start.y) / Self::CONNECT_Y_FACTOR * diff_x.min(Self::CONNECT_Y_FACTOR);
            diff_x = Self::CONNECT_MIN_X + (diff_x * Self::CONNECT_NEGATIVE_X_FACTOR).min(Self::CONNECT_MAX_X);
        } else {
            diff_x *= Self::CONNECT_POSITIVE_X_FACTOR;
        }
        diff_x = diff_x.max(Self::CONNECT_MIN_X);
        [
            start,
            egui::pos2(start.x + diff_x, start.y + diff_y),
            egui::pos2(end.x - diff_x, end.y - diff_y),
            end
        ]
    }

    ///draws the connection between two points
    fn draw_connection(painter: &egui::Painter, start: egui::Pos2, end: egui::Pos2) {
        let connection = CubicBezierShape::from_points_stroke(
            Self::get_connection_points(start, end),
            false,
            egui::Color32::TRANSPARENT,
            egui::Stroke::new(Self::CONNECT_THICKNESS, PortUi::FILLED_COLOR)
        );
        painter.circle_filled(start, PortUi::FILLED_RADIUS, PortUi::FILLED_COLOR);
        painter.circle_filled(end, PortUi::FILLED_RADIUS, PortUi::FILLED_COLOR);
        painter.add(connection);
    }
}

impl eframe::App for StarshipApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                if ui.button("Quit").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                ui.add_space(16.0);

                egui::warn_if_debug_build(ui);
            });
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

                //Draw Connections
                {
                    let painter = ui.painter();

                    //Draw existing connections
                    for connection in &self.connections {
                        let src = connection.src();
                        let dst = connection.dst();
                        let start = port_positions[&src];
                        let end = port_positions[&dst];
                        Self::draw_connection(&painter, start, end);
                    }

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
                            Self::draw_connection(painter, start, end);
                        } else {
                            self.connection_proposal.cancel();
                        }

                    } else if let ConnectionProposalState::Finalized(start, end) = *self.connection_proposal.state() {
                        self.add_connection(ConnectionId::new(start, end));
                        Self::draw_connection(
                            painter,
                            port_positions[&start],
                            port_positions[&end]
                        );
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
