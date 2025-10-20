use std::{sync::Arc, time::Instant};

use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}, Device, Host, Stream, SupportedStreamConfig};
use eframe;
use egui::{
    Align, CentralPanel, ComboBox, Context, FontData, FontDefinitions, FontFamily, Id, Label, MenuBar, Modal, RichText, TextStyle, TextWrapMode, TopBottomPanel, Ui, ViewportCommand
};

use crate::{
    circuit::{CircuitBuilderSpecification, CircuitUiSlot}, patch::PatchEditor
};

#[derive(Debug, PartialEq, Eq)]
enum AppMode {
    Editor,
    StartPlayback,
    Playback,
    EndPlayback,
}

pub struct App<'a> {
    patch_editor: PatchEditor<'a>,

    // io configuration ui
    host: Host,
    output_device: Option<Device>,
    output_device_config: Option<SupportedStreamConfig>,
    known_output_devices: Vec<Device>,
    draw_settings_ui: bool,

    // playback data
    circuit_uis: Vec<CircuitUiSlot>,
    stream: Option<Stream>,
    
    // misc
    mode: AppMode,
}

impl<'a> App<'a> {
    const MIN_ZOOM: f32 = 0.25;
    const MAX_ZOOM: f32 = 1.0;

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

        //setup audio
        let host = cpal::default_host();
        let output_device = host.default_output_device()
            .expect("No output device available.");

        let output_device_config = output_device.default_output_config()
            .expect("Default config not found.");

        let known_output_devices = {
            let iter_raw = host.output_devices();
            if let Ok(iter) = iter_raw {
                iter.collect()
            } else {
                Vec::new()
            }
        };

        // Return initialized state
        Self {
            patch_editor: PatchEditor::new(builders),

            stream: None,
            circuit_uis: Vec::new(),
            mode: AppMode::Editor,

            host,
            output_device: Some(output_device),
            output_device_config: Some(output_device_config),
            known_output_devices,
            draw_settings_ui: false
        }
    }

    pub fn begin_playback(&mut self) {
        println!(
            "Starting playback on '{}' with sample format {}.",
            self.output_device
                .as_ref()
                .expect("no output device")
                .name()
                .unwrap_or("N/A".to_string()),
            self.output_device
                .as_ref()
                .unwrap()
                .default_output_config()
                .unwrap()
                .sample_format()
        );

        let error_callback = |err| eprintln!("an error occurred on the output audio stream: {}", err);

        let sample_rate = self.output_device_config
            .as_ref()
            .expect("no device config")
            .sample_rate();
        let sample_format = self.output_device_config
            .as_ref()
            .unwrap()
            .sample_format();

        //setup backend data
        let build_backend_start = Instant::now();
        let (backend_data, frontend_data) = self.patch_editor.playback_data(
            sample_rate.0,
            crate::constants::SAMPLE_MULTIPLIER
        );
        let build_backend_end = Instant::now();

        let config_copy = self.output_device_config.clone().unwrap();

        let build_stream_start = Instant::now();
        let stream = backend_data.into_output_stream(
            self.output_device.as_ref().unwrap(),
            &config_copy.into(),
            error_callback,
            None,
            sample_format,
            sample_rate
        ).expect("Audio stream could not be built.");
        let build_stream_end = Instant::now();

        println!(
            "Backend Build Duration: {} ms\nStream Build Duration: {} ms",
            (build_backend_end - build_backend_start).as_secs_f64() * 1000.0,
            (build_stream_end - build_stream_start).as_secs_f64() * 1000.0,
        );

        let _ = stream.play();
        self.stream = Some(stream);
        self.circuit_uis = frontend_data;
    }

    pub fn end_playback(&mut self) {
        self.stream = None;
        self.circuit_uis = Vec::new();
    }

    fn draw_io_configuration_ui(
        &mut self,
        ui: &mut Ui
    ) {
        let title = RichText::new("Audio Settings").text_style(TextStyle::Heading);
        ui.add(Label::new(title).wrap());
        ui.separator();

        let mut new_select_index = None;
        ui.horizontal(|ui| {
            ui.label("Audio Output Device");
            let selected_device_text = format!(
                "{}",
                if let Some(device) = self.output_device.as_ref() {
                    device.name().unwrap_or("[No Name]".to_string())
                } else {
                    "[None]".to_string()
                }
            );
            ComboBox::from_id_salt("audio device")
                .selected_text(selected_device_text)
                .show_ui(ui, |ui| {
                    for (i, device) in self.known_output_devices.iter().enumerate() {
                        ui.selectable_value(
                            &mut new_select_index,
                            Some(i),
                            device.name().unwrap_or("[No Name]".to_string())
                        );
                    }
                });
        });

        if let Some(selected) = new_select_index {
            self.output_device = Some(self.known_output_devices[selected].clone());
        }

        ui.separator();
    }

    fn draw_editor_mode(&mut self, ctx: &Context) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            MenuBar::new().ui(ui, |ui| {
                if ui.button("Settings").clicked() {
                    self.draw_settings_ui = true;
                }

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

        if self.draw_settings_ui {
            Modal::new(Id::new("settings"))
                .show(ctx, |ui| {
                    self.draw_io_configuration_ui(ui);
                    ui.vertical_centered(|ui| {
                        if ui.button("Close").clicked() {
                            self.draw_settings_ui = false;
                        }
                    })
                });
        }

        CentralPanel::default()
            .show(&ctx, |ui| {
                self.patch_editor.draw(ui);
            });

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

impl eframe::App for App<'_> {
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
// - See connection_builder, write specificationwrapper class to handle special cases
// - Add ability to select audio host
// - Add error handling for devices being unavailable.
// - Add ability to modify stream configuration
// - Add ability to save/load states
// - Add ability to select/configure audio device before starting playback
// - Add mouse coordinates, zoom to editor
// - Clean up inspector ui
// - Make ports highlighted when focused
// - Make it so that when hovering a delete connection button,
//   the connection/connected port is highlighted
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

