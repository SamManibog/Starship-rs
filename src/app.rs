use std::{collections::HashMap, sync::Arc};

use eframe;
use egui::{
    CentralPanel,
    Context,
    Color32,
    Id,
    MenuBar,
    Pos2,
    Response,
    Sense,
    SidePanel,
    TextWrapMode,
    TopBottomPanel,
    Ui,
    Vec2,
    ViewportCommand,
};

use crate::{
    circuit::CircuitBuilderFrontend, 
    circuit_id::{
        CircuitId,
        CircuitPortId,
        ConnectionId,
        PortKind
    },
    circuits::TestCircuitBuilder,
    connection_manager::ConnectionManager,
    connection_proposal::{
        ConnectionProposal,
        ConnectionProposalState
    }
};

#[derive(Debug)]
enum Drag {
    SceneDrag(Vec2),
    ModuleDrag(CircuitId, Vec2),
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
    builder_ids: Vec<CircuitId>,
    builder_map: HashMap<CircuitId, CircuitBuilderFrontend>,
    builder_pos_map: HashMap<CircuitId, Pos2>,
    connections: ConnectionManager,
    connection_proposal: ConnectionProposal,
    focused_port: Option<CircuitPortId>,
}

impl Default for StarshipApp {
    fn default() -> Self {
        let mut output = Self {
            cam_pos: egui::vec2(0.0, 0.0),
            builder_ids: vec![],
        	builder_map: HashMap::new(),
            builder_pos_map: HashMap::new(),
            connections: Default::default(),
            connection_proposal: Default::default(),
            focused_port: None,
        };
        output.add_circuit_builder(
            CircuitBuilderFrontend::new(
                unsafe { CircuitId::new() },
                Box::new(TestCircuitBuilder::new())
            ),
            egui::pos2(100.0, 100.0)
        );
        output.add_circuit_builder(
            CircuitBuilderFrontend::new(
                unsafe { CircuitId::new() },
                Box::new(TestCircuitBuilder::new())
            ),
            egui::pos2(200.0, 200.0)
        );
        output
    }
}

impl StarshipApp {
    pub const CONNECT_COLORS: &[Color32] = &[
        Color32::RED,
        Color32::BLUE,
        Color32::GREEN,
        Color32::MAGENTA,
        Color32::YELLOW,
    ];

    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you cjn customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        cc.egui_ctx.set_style({
            let mut style = egui::Style::default();
            style.wrap_mode = Some(TextWrapMode::Extend);
            style.interaction.selectable_labels = false;
            Arc::new(style)
        });
        Default::default()
    }

    /// Adds a new circuit
    pub fn add_circuit_builder(&mut self, circuit: CircuitBuilderFrontend, position: Pos2) {
        self.builder_ids.push(circuit.id());
        self.builder_pos_map.insert(circuit.id(), position);
        self.builder_map.insert(circuit.id(), circuit);
    }

    /// Removes the circuit with the given id
    pub fn remove_circuit_builder(&mut self, id: CircuitId) {
        self.builder_ids.retain(|entry| *entry != id);
        self.builder_pos_map.remove(&id);
        self.builder_map.remove(&id);
        todo!();
        //must individually delete all associated connections
        //must handle case where a deleted connection is focused
    }

    /// Draws the connection editor in the given ui
    fn draw_connection_editor(&self, ui: &mut Ui) {
        if let Some(id) = self.focused_port {
            let spec = self.builder_map[&id.circuit_id()].builder().specification();
            let port_name = match id.port_id.kind() {
                PortKind::Input => spec.input_names[id.port_id.index()],
                PortKind::Output => spec.output_names[id.port_id.index()],
            };
            ui.label(format!(
                "Circuit: {}, Port: {}", 
                spec.name,
                port_name
            ));
        } else {
            ui.label("Click a port to focus it.");
        }
    }
}

impl eframe::App for StarshipApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            MenuBar::new().ui(ui, |ui| {
                if ui.button("Quit").clicked() {
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                }
                ui.add_space(16.0);

                egui::warn_if_debug_build(ui);
            });
        });


        SidePanel::right("right_panel")
            .max_width(400.0)
            .show(ctx, |ui| {
                ui.separator();
                self.draw_connection_editor(ui);
            });

        //A map CircuitPortId -> egui::Pos2
        //used to draw connections between ports
        let mut port_positions = HashMap::<CircuitPortId, Pos2>::new();

        let drag = CentralPanel::default()
            .show(ctx, |ui| {
                //check if scene is dragged
                let scene_response = ui.interact(
                    ui.max_rect(),
                    Id::new("Scene"),
                    Sense::DRAG
                );

                let mut mod_response: Option<(CircuitId, Response)> = None;
                for id in self.builder_ids.iter_mut() {
                    let response = self.builder_map.get_mut(id).unwrap().show(
                        self.builder_pos_map[id] - self.cam_pos,
                        ui,
                        &mut port_positions,
                        &mut self.connection_proposal,
                    );
                    if response.dragged() {
                        mod_response = Some((*id, response))
                    }
                }

                //Draw Connections and handle connection state
                {
                    let painter = ui.painter();

                    self.connections.draw_connections(painter, &port_positions);

                    //draw new connections and handle new connection state
                    if let ConnectionProposalState::Started(connection) = &self.connection_proposal.state() {
                        self.focused_port = Some(*connection);
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
                    } else if let ConnectionProposalState::Clicked(id) = *self.connection_proposal.state() {
                        self.focused_port = Some(id);
                        self.connection_proposal.cancel();
                    }
                }

                if let Some((id, response)) = mod_response {
                    Drag::ModuleDrag(id, response.drag_delta())
                } else if scene_response.dragged() {
                    Drag::SceneDrag(scene_response.drag_delta())
                } else {
                    Drag::NoDrag
                }
            }).inner;

        match drag {
            Drag::ModuleDrag(id, delta) => { *self.builder_pos_map.get_mut(&id).unwrap() += delta; },
            Drag::SceneDrag(delta) => { self.cam_pos -= delta; },
            Drag::NoDrag => {},
        }

    }
}
