use std::sync::Arc;

use eframe;

use crate::{circuit::CircuitBuilderFrontend, circuit_id::CircuitId, circuits::TestCircuitBuilder};

#[derive(Debug)]
pub enum Drag {
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
}

impl eframe::App for StarshipApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                if ui.button("Quit").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                ui.add_space(16.0);

                egui::warn_if_debug_build(ui);
            });
        });

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
                    let response = builder.show(self.area_positions[index] - self.cam_pos, ui);
                    if response.dragged() {
                        mod_response = Some((index, response))
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
