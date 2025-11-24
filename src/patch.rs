use std::collections::{HashSet, HashMap};

use egui::{Pos2, Ui, Label, RichText, TextStyle, Rect, Context, Frame, Sense, Area, Scene, Response, Color32, ScrollArea, Vec2, CentralPanel, SidePanel};

use crate::{
    circuit::{CircuitBuilder, CircuitBuilderSpecification, CircuitUiSlot}, circuit_id::{CircuitId, CircuitIdManager, CircuitPortId, ConnectionId, PortKind}, circuit_input::{CircuitInput, PortInputState}, circuits::{ConstantBuilder, SpecialInputBuilder, SpecialOutputBuilder}, connection_builder::ConnectionBuilder, connection_manager::ConnectionManager, playback::CompiledPatch
};

#[derive(Debug)]
pub enum InspectorFocus {
    None,
    Port(CircuitPortId),
    Circuit(CircuitId),
}

#[derive(Debug)]
pub struct Patch {
    // generates new unique ids
    id_manager: CircuitIdManager,

    // every used id of a circuit, for fast iteration
    builder_ids: Vec<CircuitId>,

    // maps a circuit id to its builder
    builder_map: HashMap<CircuitId, Box<dyn CircuitBuilder>>,

    // maps a circuit id to its connection builder
    connection_builder_map: HashMap<CircuitId, ConnectionBuilder>,

    // maps a circuit id to the position of its connection builder
    connection_builder_pos: HashMap<CircuitId, Pos2>,

    // keeps track of all connections in the patch
    connections: ConnectionManager,

    // a list of sets of ids that are special inputs/outputs
    input_ids: Vec<HashSet<CircuitId>>,
    output_ids: Vec<HashSet<CircuitId>>,

    // a list of possible special input/output names (order matters)
    inputs: Vec<String>,
    outputs: Vec<String>,
}

#[derive(Debug)]
pub struct PatchEditor<'a> {
    cam_pos: egui::Vec2,
    zoom: f32,
    circuit_input: CircuitInput,
    inspector_focus: InspectorFocus,
    draw_new_circuit_ui: Option<Pos2>,
    builders: &'a[CircuitBuilderSpecification],
    data: Patch
}

impl<'a> PatchEditor<'a> {
    const MIN_ZOOM: f32 = 0.25;
    const MAX_ZOOM: f32 = 1.0;

    pub fn new(
        builders: &'a[CircuitBuilderSpecification],
        inputs: Vec<String>,
        outputs: Vec<String>,
    ) -> Self {
        Self {
            cam_pos: egui::vec2(0.0, 0.0),
            zoom: 1.0,
            circuit_input: Default::default(),
            inspector_focus: InspectorFocus::None,
            draw_new_circuit_ui: None,
            builders,
            data: Patch::new(inputs, outputs)
        }
    }

    pub fn draw(&mut self, ui: &mut Ui) {
        SidePanel::right("right_panel")
            .max_width(300.0)
            .min_width(200.0)
            .show_inside(ui, |ui| {
                self.draw_inspector(ui);
            });


        let mut old_new_circuit_ui = self.draw_new_circuit_ui != None;

        //A map CircuitPortId -> egui::Pos2
        //used to draw connections between ports
        let mut port_positions = HashMap::<CircuitPortId, Pos2>::new();

        let mut scene_rect = Rect::NOTHING;
        let mut window_size = Vec2::ZERO;
        let mut clip_rect = Rect::NOTHING;

        CentralPanel::default().show_inside(ui, |ui| {
            window_size = ui.available_size();
            scene_rect = Rect::from_center_size(
                self.cam_pos.to_pos2(),
                window_size / self.zoom
            );
            let scene_min_pos = scene_rect.min.to_vec2();
            clip_rect = ui.response().rect;
            let response = Scene::new()
                .zoom_range(Self::MIN_ZOOM..=Self::MAX_ZOOM)
                .sense(Sense::click_and_drag())
                .show(ui, &mut scene_rect, |ui| {

                    let mut mod_response: Option<(CircuitId, Response)> = None;
                    for id in self.data.builder_ids.iter_mut() {
                        let highlight = match self.inspector_focus {
                            InspectorFocus::Port(port) => port.unit_id == *id,
                            InspectorFocus::Circuit(circuit) => circuit == *id,
                            InspectorFocus::None => false
                        };
                        let response = self.data.connection_builder_map.get_mut(id).unwrap().show(
                            self.data.connection_builder_pos[id],// - self.cam_pos,
                            ui,
                            &mut port_positions,
                            &mut self.circuit_input,
                            highlight,
                            self.data.builder_map[&id].name()
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

                        self.data.connections.draw_connections(painter, &port_positions);

                        //draw new connections and handle new connection state
                        if let PortInputState::StartConnection(connection) = &self.circuit_input.state() {
                            self.inspector_focus = InspectorFocus::Port(*connection);
                            //ensure we are still dragging and on-screen
                            let mouse_pos_opt = ui.input(|input| {
                                if input.pointer.primary_released() {
                                    None
                                } else {
                                    input.pointer.latest_pos()
                                }
                            });

                            //if mouse state is good, draw the connection
                            //otherwise, cancel the connection
                            if let Some(raw_mouse_pos) = mouse_pos_opt {
                                let mouse_pos = (raw_mouse_pos - clip_rect.min.to_vec2()) / self.zoom + scene_min_pos;
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
                            self.add_connection(start, end);
                            self.circuit_input.clear();
                        } else if let PortInputState::Click(id) = *self.circuit_input.state() {
                            self.inspector_focus = InspectorFocus::Port(id);
                            self.circuit_input.clear();
                        }
                    }

                    if ui.response().secondary_clicked() {
                        self.draw_new_circuit_ui = Some(ui.response().interact_pointer_pos().unwrap());
                        old_new_circuit_ui = false;
                    }

                    mod_response
                });

            if let Some(pos) = self.draw_new_circuit_ui {
                self.draw_new_circuit_ui(
                    ui.ctx(),
                    pos,
                    scene_rect,
                    clip_rect,
                    old_new_circuit_ui
                );
            }

            if let Some((id, inner)) = response.inner {
                *self.data.connection_builder_pos.get_mut(&id).unwrap() += inner.drag_delta();
            }
        });

        let (p_cam, p_zoom) = (self.cam_pos, self.zoom);

        self.cam_pos = scene_rect.center().to_vec2();
        self.zoom = window_size.x / (scene_rect.max.x - scene_rect.min.x);

        if p_cam != self.cam_pos || p_zoom != self.zoom {
            self.draw_new_circuit_ui = None;
        }
    }

    fn draw_new_circuit_ui(
        &mut self,
        ctx: &Context,
        position: Pos2,
        scene_rect: Rect,
        scene_clip_rect: Rect,
        old: bool
    ) {
        let true_pos = (position - scene_rect.min).to_pos2() * self.zoom + scene_clip_rect.min.to_vec2();

        let response = Area::new(egui::Id::new("new_circuit_ui"))
            .sense(Sense::click_and_drag())
            .fixed_pos(true_pos)
            .show(ctx, |ui| {
                Frame::new()
                    .fill(ui.style().visuals.window_fill)
                    .stroke(ui.style().visuals.window_stroke)
                    .inner_margin(4.0)
                    .corner_radius(2)
                    .show(ui, |ui| {
                        ui.label("Add a circuit");
                        ui.separator();
                        ScrollArea::vertical().show(ui, |ui| {
                            if ui.button("Constant").clicked() {
                                let id = self.add_constant(position);
                                self.inspector_focus = InspectorFocus::Circuit(id);
                            }
                            for builder in self.builders {
                                if ui.button(&builder.display_name).clicked() {
                                    let id = self.add_circuit_by_builder(
                                        (builder.instance)(),
                                        position
                                    );
                                    self.inspector_focus = InspectorFocus::Circuit(id);
                                }
                            }
                            let mut add_input = None;
                            for (index, input) in self.data.inputs.iter().enumerate() {
                                if ui.button(input).clicked() {
                                    add_input = Some(index);
                                }
                            }
                            let mut add_output = None;
                            for (index, output) in self.data.outputs.iter().enumerate() {
                                if ui.button(output).clicked() {
                                    add_output = Some(index);
                                }
                            }
                            if let Some(index) = add_input {
                                let id = self.add_input(index, position);
                                self.inspector_focus = InspectorFocus::Circuit(id);
                            } else if let Some(index) = add_output {
                                let id = self.add_output(index, position);
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
            self.draw_new_circuit_ui = None;
        }
    }

    fn draw_inspector(&mut self, ui: &mut Ui) {
        if let InspectorFocus::Port(id) = self.inspector_focus {
            {
                let name = self.data.builder_map[&id.circuit_id()].name();
                let spec = self.data.connection_builder_map[&id.circuit_id()].specification();
                let port_name = match id.port_id.kind() {
                    PortKind::Input => spec.input_names[id.port_id.index()],
                    PortKind::Output => spec.output_names[id.port_id.index()],
                };
                let title = RichText::new(port_name).text_style(TextStyle::Heading);
                ui.add(Label::new(title).wrap());
                ui.add(Label::new(name).wrap());
            }
            ui.separator();
            let connected_raw = self.data.connections.port_query_ports(id);
            let mut remove_connection = None;
            if let Some(connected) = connected_raw {
                for port in connected {
                    let circuit_name = self.data.builder_map[&id.circuit_id()].name();
                    let spec = self.data.connection_builder_map[&port.circuit_id()].specification();
                    let port_name = match port.port_id.kind() {
                        PortKind::Input => spec.input_names[port.port_id.index()],
                        PortKind::Output => spec.output_names[port.port_id.index()],
                    };
                    let button_text = format!(
                        "Circuit: {}, Port: {}", 
                        circuit_name,
                        port_name
                    );
                    if ui.button(button_text).clicked() {
                        remove_connection = Some(port);
                    }
                }
            }
            if let Some(connection) = remove_connection {
                self.data.connections.remove_connection(ConnectionId::new_auto(
                    *connection,
                    id
                ));
            }
        } else if let InspectorFocus::Circuit(id) = self.inspector_focus {
            let name = self.data.builder_map[&id].name();
            let title = RichText::new(name).text_style(TextStyle::Heading);
            ui.horizontal(|ui| {
                ui.label(title);
                if ui.small_button("X").clicked() {
                    self.remove_circuit_builder(id);
                }
            });
            ui.separator();
            if let Some(builder) = self.data.builder_map.get_mut(&id) {
                builder.show(ui);
            }

        } else {
            let tip = Label::new("Click a port or circuit to focus it. Right click in the central area to add a circuit.")
                .wrap();
            ui.add(tip);
        }
        ui.separator();
    }

    pub fn add_constant(&mut self, position: Pos2) -> CircuitId {
        self.data.add_constant(position)
    }

    pub fn add_input(&mut self, index: usize, position: Pos2) -> CircuitId {
        self.data.add_input(index, position)
    }

    pub fn add_output(&mut self, index: usize, position: Pos2) -> CircuitId {
        self.data.add_output(index, position)
    }

    /// Adds a new circuit at the given position
    /// Do not use this method to add a speaker circuit. Use add_speaker() instead.
    /// Returns the id of the new circuit
    pub fn add_circuit_by_builder(
        &mut self,
        circuit_builder: Box<dyn CircuitBuilder>,
        position: Pos2
    ) -> CircuitId {
        self.data.add_circuit_by_builder(circuit_builder, position)
    }

    /// Adds a connection for the two given circuit ports
    pub fn add_connection(&mut self, src: CircuitPortId, dst: CircuitPortId) {
        self.data.add_connection(src, dst)
    }

    /// Removes the circuit with the given id
    pub fn remove_circuit_builder(&mut self, id: CircuitId) {
        //unfocus connection or builder if it was deleted
        match self.inspector_focus {
            InspectorFocus::Port(focus_id) => {
                if focus_id.unit_id == id {
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

        self.data.remove_circuit_builder(id);
    }

    pub fn playback_data(
        &self,
        sample_rate: u32,
        sample_multiplier: f32
    ) -> (CompiledPatch, Vec<CircuitUiSlot>) {
        self.data.compile(sample_rate, sample_multiplier)
    }

}

impl Patch {
    pub fn new(inputs: Vec<String>, outputs: Vec<String>) -> Self {
        let input_ids = {
            let mut map = Vec::new();
            for _ in &inputs {
                map.push(HashSet::new());
            }
            map
        };
        let output_ids = {
            let mut map = Vec::new();
            for _ in &outputs {
                map.push(HashSet::new());
            }
            map
        };

        // Return initialized state
        Self {
            id_manager: Default::default(),
            builder_ids: vec![],
            builder_map: HashMap::new(),
        	connection_builder_map: HashMap::new(),
            connection_builder_pos: HashMap::new(),
            connections: Default::default(),
            input_ids,
            output_ids,
            inputs,
            outputs
        }
    }

    pub fn inputs(&self) -> &[String] {
        &self.inputs
    }

    pub fn outputs(&self) -> &[String] {
        &self.outputs
    }

	pub fn add_constant(&mut self, position: Pos2) -> CircuitId {
        let id = self.id_manager.get_id();
        let builder = Box::new(ConstantBuilder::new());
        let frontend = ConnectionBuilder::new_constant(id, builder.data());
        self.add_circuit(builder, frontend, position);
        id
    }

    /// Convenience method. Adds a new input circuit at the given position by its index in
    /// self.inputs.
    pub fn add_input(&mut self, index: usize, position: Pos2) -> CircuitId {
        debug_assert!(index < self.inputs.len(), "Index must be <= the number of allowed inputs.");
        let id = self.id_manager.get_id();
        let name = self.inputs[index].clone();
        let builder = Box::new(SpecialInputBuilder::new(name.clone()));
        let frontend = ConnectionBuilder::new_special_input(id, name);
        self.add_circuit(builder, frontend, position);
        id
    }

    /// Convenience method. Adds a new output circuit at the given position by its index in
    /// self.outputs.
    pub fn add_output(&mut self, index: usize, position: Pos2) -> CircuitId {
        debug_assert!(index < self.outputs.len(), "Index must be <= the number of allowed inputs.");
        let id = self.id_manager.get_id();
        let name = self.outputs[index].clone();
        let builder = Box::new(SpecialOutputBuilder::new(name.clone()));
        let frontend = ConnectionBuilder::new_special_output(id, name);
        self.add_circuit(builder, frontend, position);
        id
    }

    /// Convenience method. Adds a new circuit at the given position
    /// Do not use this method to add input or output circuits. Use add_input()/add_output().
    /// Do not use this method to add a constant circuit. Use add_constant().
    /// Returns the id of the new circuit
    pub fn add_circuit_by_builder(
        &mut self,
        circuit_builder: Box<dyn CircuitBuilder>,
        position: Pos2
    ) -> CircuitId {
        let id = self.id_manager.get_id();
        let frontend = ConnectionBuilder::new(id, circuit_builder.specification());
        self.add_circuit(circuit_builder, frontend, position);
        id
    }

    /// Adds the circuit's associated builder and connection builder to the patch at the given position
    pub fn add_circuit(
        &mut self,
        circuit_builder: Box<dyn CircuitBuilder>,
        connection_builder: ConnectionBuilder,
        position: Pos2
    ) {
        self.builder_map.insert(connection_builder.id(), circuit_builder);
        self.builder_ids.push(connection_builder.id());
        self.connection_builder_pos.insert(connection_builder.id(), position);
        self.connection_builder_map.insert(connection_builder.id(), connection_builder);
    }

    /// Adds a connection for the two given circuit ports
    pub fn add_connection(&mut self, src: CircuitPortId, dst: CircuitPortId) {
        self.connections.add_connection(ConnectionId::new(src, dst));
    }

    /// Removes the circuit with the given id
    pub fn remove_circuit_builder(&mut self, id: CircuitId) {
        self.builder_ids.retain(|entry| *entry != id);
        self.builder_map.remove(&id);
        self.connection_builder_pos.remove(&id);
        self.connection_builder_map.remove(&id);
        self.connections.remove_circuit(id);
        
        // remove circuit from input, output ids
        for set in self.input_ids.iter_mut() {
            set.remove(&id);
        }
        for set in self.output_ids.iter_mut() {
            set.remove(&id);
        }
    }

    /// Creates the playback data for the patch
    pub fn compile(
        &self,
        sample_rate: u32,
        sample_multiplier: f32
    ) -> (CompiledPatch, Vec<CircuitUiSlot>) {
        CompiledPatch::new(
            &self.builder_ids,
            &self.builder_map,
            &self.connections,
            &self.input_ids,
            &self.output_ids,
            sample_rate,
            sample_multiplier
        )
    }
}
