use std::{collections::{HashMap, HashSet}, sync::Arc};

use eframe;
use egui::{
    Area, CentralPanel, Color32, Context, Frame, Id, MenuBar, Pos2, Response, ScrollArea, Sense, SidePanel, TextWrapMode, TopBottomPanel, Ui, Vec2, ViewportCommand
};

use crate::{
    circuit::{CircuitBuilder, CircuitBuilderFrontend, CircuitBuilderSpecification}, circuit_id::{ CircuitId, CircuitPortId, ConnectionId, PortKind }, circuit_input::{ CircuitInput, PortInputState }, circuits::SpeakerBuilder, connection_manager::ConnectionManager
};

#[derive(Debug)]
enum CentralInput {
    SceneDrag(Vec2),
    ModuleDrag(CircuitId, Vec2),
    SceneRightClick(Pos2),
    NoInput
}

#[derive(Debug)]
enum InspectorFocus {
    None,
    Port(CircuitPortId),
    Circuit(CircuitId),
}

impl Default for CentralInput {
    fn default() -> Self {
        Self::NoInput
    }
}

#[derive(Debug)]
pub struct StarshipApp<'a> {
    cam_pos: egui::Vec2,
    builder_ids: Vec<CircuitId>,
    builder_map: HashMap<CircuitId, CircuitBuilderFrontend>,
    builder_pos_map: HashMap<CircuitId, Pos2>,
    speakers: HashSet<CircuitId>,
    connections: ConnectionManager,
    circuit_input: CircuitInput,
    inspector_focus: InspectorFocus,
    builders: &'a[CircuitBuilderSpecification],
    new_circuit_ui: Option<Pos2>,
}

impl<'a> StarshipApp<'a> {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>, builders: &'a[CircuitBuilderSpecification]) -> Self {
        // This is also where you cjn customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        cc.egui_ctx.set_style({
            let mut style = egui::Style::default();
            style.wrap_mode = Some(TextWrapMode::Extend);
            style.interaction.selectable_labels = false;
            Arc::new(style)
        });
        Self {
            cam_pos: egui::vec2(0.0, 0.0),
            builder_ids: vec![],
        	builder_map: HashMap::new(),
            builder_pos_map: HashMap::new(),
            speakers: HashSet::new(),
            connections: Default::default(),
            circuit_input: Default::default(),
            inspector_focus: InspectorFocus::None,
            builders,
            new_circuit_ui: None,
        }
    }

    /// Adds a new speaker at the given position
    /// Returns the id of the new speaker
	pub fn add_speaker(&mut self, position: Pos2) -> CircuitId {
        let builder = Box::new(SpeakerBuilder::new());
        let id = self.add_circuit_builder(builder, position);
        self.speakers.insert(id);
        id
    }

    /// Adds a new circuit at the given position
    /// Do not use this method to add a speaker circuit. Use add_speaker() instead.
    /// Returns the id of the new circuit
    pub fn add_circuit_builder(
        &mut self,
        circuit_builder: Box<dyn CircuitBuilder>,
        position: Pos2
    ) -> CircuitId {
        let id = unsafe { CircuitId::new() };
        let frontend = CircuitBuilderFrontend::new(id, circuit_builder);
        self.builder_ids.push(frontend.id());
        self.builder_pos_map.insert(frontend.id(), position);
        self.builder_map.insert(frontend.id(), frontend);
        id
    }

    /// Removes the circuit with the given id
    pub fn remove_circuit_builder(&mut self, id: CircuitId) {
        //unfocus connection or builder if it was deleted
        match self.inspector_focus {
            InspectorFocus::Port(focus_id) => {
                if focus_id.circuit_id == id {
                    self.inspector_focus = InspectorFocus::None;
                }
            }
            InspectorFocus::Circuit(focus_id) => {
                if focus_id == id {
                    self.inspector_focus = InspectorFocus::None;
                }
            }
            InspectorFocus::None => {}
        }

        //delete builder
        self.builder_ids.retain(|entry| *entry != id);
        self.builder_pos_map.remove(&id);
        self.builder_map.remove(&id);
        self.speakers.remove(&id);
        self.connections.remove_circuit(id);
    }

    /// Draws the ui for adding a new circuit at the given location
    fn draw_new_circuit_ui(&mut self, ctx: &Context, position: Pos2, old: bool) {
        let response = Area::new(Id::new("new_circuit_ui"))
            .fixed_pos(position)
            .sense(Sense::click_and_drag())
            .show(ctx, |ui| {
                Frame::new() .show(ui, |ui| {
                    ui.label("Add a circuit...");
                    ScrollArea::vertical().show(ui, |ui| {
                        for builder in self.builders {
                            if ui.button(&builder.display_name).clicked() {
                                let id = self.add_circuit_builder(
                                    (builder.instance)(),
                                    position + self.cam_pos
                                );
                                self.inspector_focus = InspectorFocus::Circuit(id);
                            }
                        }
                        if ui.button("Speaker").clicked() {
                            let id = self.add_speaker(position + self.cam_pos);
                            self.inspector_focus = InspectorFocus::Circuit(id);
                        }
                    });
                })
            }).response;

        // If there was some click off of the ui, close it
        // If there was a click on one of the buttons, will cancel too
        if old && !response.clicked() && ctx.input(|i| {
            i.pointer.any_click() || i.pointer.is_decidedly_dragging()
        }) {
            self.new_circuit_ui = None;
        }
    }

    /// Draws the inspector to the given ui
    fn draw_inspector(&mut self, ui: &mut Ui) {
        if let InspectorFocus::Port(id) = self.inspector_focus {
            {
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
            }
            let connected_raw = self.connections.query_connected(id);
            let mut remove_connection = None;
            if let Some(connected) = connected_raw {
                for port in connected {
                    let spec = self.builder_map[&port.circuit_id()].builder().specification();
                    let port_name = match port.port_id.kind() {
                        PortKind::Input => spec.input_names[port.port_id.index()],
                        PortKind::Output => spec.output_names[port.port_id.index()],
                    };
                    let button_text = format!(
                        "Circuit: {}, Port: {}", 
                        spec.name,
                        port_name
                    );
                    if ui.button(button_text).clicked() {
                        remove_connection = Some(port);
                    }
                }
            }
            if let Some(connection) = remove_connection {
                self.connections.remove_connection(ConnectionId::new_auto(
                    *connection,
                    id
                ));
            }
        } else if let InspectorFocus::Circuit(id) = self.inspector_focus {
            if ui.button(format!("Delete {:?}", id)).clicked() {
                self.remove_circuit_builder(id);
            }
        } else {
            ui.label("Click a port or circuit to focus it.");
        }
        ui.separator();
    }

}

impl eframe::App for StarshipApp<'_>{
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
                self.draw_inspector(ui);
            });

        let old_new_circuit_ui = self.new_circuit_ui != None;

        //A map CircuitPortId -> egui::Pos2
        //used to draw connections between ports
        let mut port_positions = HashMap::<CircuitPortId, Pos2>::new();

        let central_response = CentralPanel::default()
            .show(ctx, |ui| {

                //check if scene is dragged
                let scene_response = ui.interact(
                    ui.max_rect(),
                    Id::new("Scene"),
                    Sense::click_and_drag()
                );

                let mut mod_response: Option<(CircuitId, Response)> = None;
                for id in self.builder_ids.iter_mut() {
                    let highlight = match self.inspector_focus {
                        InspectorFocus::Port(port) => port.circuit_id == *id,
                        InspectorFocus::Circuit(circuit) => circuit == *id,
                        InspectorFocus::None => false
                    };
                    let response = self.builder_map.get_mut(id).unwrap().show(
                        self.builder_pos_map[id] - self.cam_pos,
                        ui,
                        &mut port_positions,
                        &mut self.circuit_input,
                        highlight
                    );
                    if response.dragged() || response.clicked() {
                        self.inspector_focus = InspectorFocus::Circuit(*id);
                    }
                    if response.dragged() {
                        mod_response = Some((*id, response))
                    }
                }

                //Draw Connections and handle connection state
                {
                    let painter = ui.painter();

                    self.connections.draw_connections(painter, &port_positions);

                    //draw new connections and handle new connection state
                    if let PortInputState::StartConnection(connection) = &self.circuit_input.state() {
                        self.inspector_focus = InspectorFocus::Port(*connection);
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
                            self.circuit_input.clear();
                        }

                    } else if let PortInputState::FinalizeConnection(start, end) = *self.circuit_input.state() {
                        self.connections.add_connection(ConnectionId::new(start, end));
                        self.circuit_input.clear();
                    } else if let PortInputState::Click(id) = *self.circuit_input.state() {
                        self.inspector_focus = InspectorFocus::Port(id);
                        self.circuit_input.clear();
                    }
                }

                if scene_response.clicked() {
                    self.inspector_focus =  InspectorFocus::None;
                }

                if let Some((id, response)) = mod_response {
                    CentralInput::ModuleDrag(id, response.drag_delta())
                } else if scene_response.dragged() {
                    CentralInput::SceneDrag(scene_response.drag_delta())
                } else if scene_response.secondary_clicked() {
                    CentralInput::SceneRightClick(scene_response.interact_pointer_pos().unwrap())
                } else {
                    CentralInput::NoInput
                }
            });

        match central_response.inner {
            CentralInput::ModuleDrag(id, delta) => { *self.builder_pos_map.get_mut(&id).unwrap() += delta; }
            CentralInput::SceneDrag(delta) => { self.cam_pos -= delta; }
            CentralInput::SceneRightClick(pos) => { self.new_circuit_ui = Some(pos); }
            CentralInput::NoInput => {}
        }

        if let Some(pos) = self.new_circuit_ui {
            self.draw_new_circuit_ui(ctx, pos, old_new_circuit_ui);
        }
    }
}
