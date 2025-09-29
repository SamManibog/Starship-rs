use std::{collections::{HashMap, HashSet}, sync::Arc};

use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}, Stream};
use eframe;
use egui::{
    Align, Area, CentralPanel, Color32, Context, FontData, FontDefinitions, FontFamily, Frame, Id, Label, MenuBar, Pos2, Response, RichText, ScrollArea, Sense, SidePanel, TextStyle, TextWrapMode, TopBottomPanel, Ui, Vec2, ViewportCommand
};

use crate::{
    circuit::{CircuitBuilder, CircuitBuilderSpecification, CircuitUiSlot}, circuit_id::{ CircuitId, CircuitPortId, ConnectionId, PortKind }, circuit_input::{ CircuitInput, PortInputState }, circuits::SpeakerBuilder, connection_builder::ConnectionBuilder, connection_manager::ConnectionManager, playback::PlaybackBackendData
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

#[derive(Debug, PartialEq, Eq)]
enum AppMode {
    Editor,
    StartPlayback,
    Playback,
    EndPlayback,
}

impl Default for CentralInput {
    fn default() -> Self {
        Self::NoInput
    }
}

pub struct App<'a> {
    // circuit functionality
    builders: &'a[CircuitBuilderSpecification],
    builder_ids: Vec<CircuitId>,
    builder_map: HashMap<CircuitId, Box<dyn CircuitBuilder>>,
    connection_builder_map: HashMap<CircuitId, ConnectionBuilder>,
    connection_builder_pos: HashMap<CircuitId, Pos2>,
    speakers: HashSet<CircuitId>,
    connections: ConnectionManager,

    // editor ui
    cam_pos: egui::Vec2,
    circuit_input: CircuitInput,
    inspector_focus: InspectorFocus,
    new_circuit_ui: Option<Pos2>,

    // playback data
    circuit_uis: Vec<CircuitUiSlot>,
    stream: Option<Stream>,
    
    // misc
    mode: AppMode,
}

impl<'a> App<'a> {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>, builders: &'a[CircuitBuilderSpecification]) -> Self {

        // Add font to handle music glyphs
        let mut fonts = FontDefinitions::default();
        let font_name = "NotoMusic";
        fonts.font_data.insert(
            font_name.to_string(),
            Arc::new(FontData::from_static(
                include_bytes!("../assets/NotoMusicModified.otf")
            )),
        );
        fonts.families.get_mut(&FontFamily::Proportional).unwrap()
            .push(font_name.to_string());
        cc.egui_ctx.set_fonts(fonts);

        // Customize egui style
        cc.egui_ctx.set_style({
            let mut style = egui::Style::default();
            style.wrap_mode = Some(TextWrapMode::Extend);
            style.interaction.selectable_labels = false;
            Arc::new(style)
        });

        // Return initialized state
        Self {
            builders,
            builder_ids: vec![],
            builder_map: HashMap::new(),
        	connection_builder_map: HashMap::new(),
            connection_builder_pos: HashMap::new(),
            speakers: HashSet::new(),
            connections: Default::default(),
            cam_pos: egui::vec2(0.0, 0.0),
            circuit_input: Default::default(),
            inspector_focus: InspectorFocus::None,
            new_circuit_ui: None,
            stream: None,
            circuit_uis: Vec::new(),
            mode: AppMode::Editor,
        }
    }

    pub fn begin_playback(&mut self) {
        //setup backend data
        let (backend_data, frontend_data) = PlaybackBackendData::new(
            &self.builder_ids,
            &self.builder_map,
            &self.connections,
            &self.speakers,
            crate::constants::SAMPLE_MULTIPLIER
        );

        //setup audio
        let host = cpal::default_host();
        let device = host.default_output_device().expect("No output device available.");
        let default_config = device.default_output_config().expect("Default config not found.");

        println!(
            "Starting playback on '{}' with sample format {}.",
            device.name().unwrap_or("N/A".to_string()),
            device.default_output_config().unwrap().sample_format()
        );

        let error_callback = |err| eprintln!("an error occurred on the output audio stream: {}", err);

        let sample_rate = default_config.sample_rate();
        let sample_format = default_config.sample_format();

        let stream = backend_data.into_output_stream(
            &device,
            &default_config.into(),
            error_callback,
            None,
            sample_format,
            sample_rate
        ).expect("Audio stream could not be built.");
        let _ = stream.play();
        self.stream = Some(stream);
        self.circuit_uis = frontend_data;
    }

    pub fn end_playback(&mut self) {
        self.stream = None;
        self.circuit_uis = Vec::new();
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
        let frontend = ConnectionBuilder::new(id, circuit_builder.specification());
        self.builder_map.insert(frontend.id(), circuit_builder);
        self.builder_ids.push(frontend.id());
        self.connection_builder_pos.insert(frontend.id(), position);
        self.connection_builder_map.insert(frontend.id(), frontend);
        id
    }

    /// Adds a connection for the two given circuit ports
    pub fn add_connection(&mut self, src: CircuitPortId, dst: CircuitPortId) {
        self.connections.add_connection(ConnectionId::new(src, dst));
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
        self.builder_map.remove(&id);
        self.connection_builder_pos.remove(&id);
        self.connection_builder_map.remove(&id);
        self.speakers.remove(&id);
        self.connections.remove_circuit(id);
    }

    /// Draws the ui for adding a new circuit at the given location
    fn draw_new_circuit_ui(&mut self, ctx: &Context, position: Pos2, old: bool) {
        let response = Area::new(Id::new("new_circuit_ui"))
            .fixed_pos(position)
            .sense(Sense::click_and_drag())
            .show(ctx, |ui| {
                Frame::new()
                    .fill(ctx.style().visuals.window_fill)
                    .stroke(ctx.style().visuals.window_stroke)
                    .inner_margin(4.0)
                    .corner_radius(2)
                    .show(ui, |ui| {
                        ui.label("Add a circuit");
                        ui.separator();
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
                let name = self.builder_map[&id.circuit_id()].name();
                let spec = self.connection_builder_map[&id.circuit_id()].specification();
                let port_name = match id.port_id.kind() {
                    PortKind::Input => spec.input_names[id.port_id.index()],
                    PortKind::Output => spec.output_names[id.port_id.index()],
                };
                let title = RichText::new(port_name).text_style(TextStyle::Heading);
                ui.add(Label::new(title).wrap());
                ui.add(Label::new(name).wrap());
            }
            ui.separator();
            let connected_raw = self.connections.port_query_ports(id);
            let mut remove_connection = None;
            if let Some(connected) = connected_raw {
                for port in connected {
                    let circuit_name = self.builder_map[&id.circuit_id()].name();
                    let spec = self.connection_builder_map[&port.circuit_id()].specification();
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
                self.connections.remove_connection(ConnectionId::new_auto(
                    *connection,
                    id
                ));
            }
        } else if let InspectorFocus::Circuit(id) = self.inspector_focus {
            let name = self.builder_map[&id].name();
            let title = RichText::new(name).text_style(TextStyle::Heading);
            ui.horizontal(|ui| {
                ui.label(title);
                if ui.small_button("X").clicked() {
                    self.remove_circuit_builder(id);
                }
            });
            ui.separator();
            if let Some(builder) = self.builder_map.get_mut(&id) {
                builder.show(ui);
            }

        } else {
            let tip = Label::new("Click a port or circuit to focus it. Right click in the central area to add a circuit.")
                .wrap();
            ui.add(tip);
        }
        ui.separator();
    }

    fn draw_editor_mode(&mut self, ctx: &Context) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            MenuBar::new().ui(ui, |ui| {
                if ui.button("Quit").clicked() {
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                }
                ui.add_space(16.0);
                egui::warn_if_debug_build(ui);
                
                //add play button to far right edge
                ui.with_layout(egui::Layout::right_to_left(Align::Max),
                    |ui| {
                        if ui.button("Play").clicked() {
                            self.mode = AppMode::StartPlayback;
                        }
                    }
                );

            });
        });

        SidePanel::right("right_panel")
            .max_width(300.0)
            .min_width(200.0)
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
                    let response = self.connection_builder_map.get_mut(id).unwrap().show(
                        self.connection_builder_pos[id] - self.cam_pos,
                        ui,
                        &mut port_positions,
                        &mut self.circuit_input,
                        highlight,
                        self.builder_map[&id].name()
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
                        self.add_connection(start, end);
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
            CentralInput::ModuleDrag(id, delta) => { *self.connection_builder_pos.get_mut(&id).unwrap() += delta; }
            CentralInput::SceneDrag(delta) => { self.cam_pos -= delta; }
            CentralInput::SceneRightClick(pos) => { self.new_circuit_ui = Some(pos); }
            CentralInput::NoInput => {}
        }

        if let Some(pos) = self.new_circuit_ui {
            self.draw_new_circuit_ui(ctx, pos, old_new_circuit_ui);
        }
    }

    fn draw_playback_mode(&mut self, ctx: &Context) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            MenuBar::new().ui(ui, |ui| {
                if ui.button("Quit").clicked() {
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                }
                ui.add_space(16.0);

                egui::warn_if_debug_build(ui);

                //add stop button to far right edge
                ui.with_layout(egui::Layout::right_to_left(Align::Max),
                    |ui| {
                        if ui.button("Stop").clicked() {
                            self.end_playback();
                            self.mode = AppMode::EndPlayback;
                        }
                    }
                );
            });
        });

        // todo this is a temporary solution
        CentralPanel::default()
            .show(ctx, |ui| {
                ui.with_layout(ui.layout().with_main_wrap(true), |ui| {
                    for circuit_ui in self.circuit_uis.iter_mut() {
                        circuit_ui.show(ui)
                    }
                })
            });

    }
}

impl eframe::App for App<'_>{
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // handle transition states
        if self.mode == AppMode::StartPlayback {
            self.begin_playback();
            self.mode = AppMode::Playback;
        } else if self.mode == AppMode::EndPlayback {
            self.end_playback();
            self.mode = AppMode::Editor;
        }

        // run main states
        match self.mode {
            AppMode::Editor => self.draw_editor_mode(ctx),
            AppMode::Playback => self.draw_playback_mode(ctx),
            _ => unreachable!()
        }
    }
}

// Todo:
// - Make sample rate a part of build state
// - Clean up inspector ui
// - Make ports highlighted when focused
// - Make it so that when hovering a delete connection button,
//   the connection/connected port is highlighted
// - Add ability to zoom
// - Resolve unbounded space to place circuits
//   - Make a hard limit on world size
//   - Add ability to select and move multiple circuits at once
//   - Add abiility to jump to groups of circuits
//   - Add coordinate display
// - Add ability for builders to have descriptions
// - Add flags field to circuit builder specification, so that
//   they may be organized in new circuit menu
// - Add menu to edit layout of controls
// - double check safety of unwrap methods

