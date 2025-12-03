use egui::{Frame, Pos2, Rect, Sense, Stroke, UiBuilder, Vec2, Window};
use starship_rust::{sequencers::curve::{Curve, CurvePointId}, utils};

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Starship Curve Editor",
        native_options,
        Box::new(|cc| {
            Ok(Box::new(
                CurveEditor::new(cc)
            ))
        })
    )
}

#[derive(Debug)]
enum EditState {
    /// no editing in progress
    Viewing,

    /// might start dragging point around screen
    PreMoving(CurvePointId),

    /// dragging a point around the screen
    Moving(CurvePointId),

    /// using context menu for the given point
    /// menu will attempt being drawn on the given curve coords
    Configuring(CurvePointId, PointConfigMenu),
}

#[derive(Debug)]
struct PointConfigMenu {
    time_text: String,
    value_text: String,
}

impl EditState {
    pub fn is_viewing(&self) -> bool {
        matches!(*self, Self::Viewing)
    }

    pub fn is_premoving(&self) -> bool {
        matches!(*self, Self::PreMoving(_))
    }

    pub fn is_moving(&self) -> bool {
        matches!(*self, Self::Moving(_))
    }

    pub fn is_moving_point(&self, point: CurvePointId) -> bool {
        if let Self::Moving(pt) = self && *pt == point {
            true
        } else {
            false
        }
    }

    pub fn is_configuring(&self) -> bool {
        matches!(*self, Self::Configuring(_, _))
    }

    pub fn is_configuring_point(&self, point: CurvePointId) -> bool {
        if let Self::Configuring(pt, _) = self && *pt == point {
            true
        } else {
            false
        }
    }
}

struct CurveEditor {
    curve: Curve,
    edit_state: EditState,
    saved_mouse_pos: Pos2,
}

impl CurveEditor {
    const POINT_RADIUS: f32 = 6.0;
    const POINT_COLOR: egui::Color32 = egui::Color32::WHITE;
    const FOCUS_POINT_COLOR: egui::Color32 = egui::Color32::RED;

    pub const MIN_WIDTH: f32 = 200.0;
    pub const MIN_HEIGHT: f32 = 200.0;

    const CONFIG_WIDTH: f32 = 100.0;
    const CONFIG_HEIGHT: f32 = 100.0;
    const CONFIG_X_OFFSET: f32 = 10.0;
    const CONFIG_Y_OFFSET: f32 = 10.0;

    const POPUP_PADDING: f32 = 20.0;

    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut curve = Curve::new(0.5, 1.0);
        curve.add_point(0.2);
        curve.add_point(0.3);
        curve.set_point_value(curve.get_nearest_point(0.3), 0.2);
        curve.add_point(0.5);
        curve.set_point_value(curve.get_nearest_point(0.5), 0.2);
        curve.add_point(0.7);
        curve.set_point_value(curve.get_nearest_point(0.7), 0.5);
        Self {
            curve,
            edit_state: EditState::Viewing,
            saved_mouse_pos: Pos2::ZERO
        }
    }

    pub fn draw(&mut self, ui: &mut egui::Ui) {
        let request_dim = {
            let available = ui.available_size();
            Vec2::new(available.x.max(Self::MIN_WIDTH), available.y.max(Self::MIN_HEIGHT))
        };
        let (response, painter) = ui.allocate_painter(request_dim, egui::Sense::click_and_drag());
        let curve_rext = response.rect;
        let curve_x_dim = curve_rext.size().x;
        let curve_y_dim = curve_rext.size().y;

        //painter.rect_stroke(curve_rext, 0.0, Stroke::new(2.0, egui::Color32::RED), egui::StrokeKind::Inside);

        // mouse/interaction position relative to current ui
        let mouse_pos =  ui.input(|input| {
            if let Some(pos) = input.pointer.latest_pos() {
                self.saved_mouse_pos = pos;
                pos
            } else {
                self.saved_mouse_pos
            }
        });

        // transformations to curve coordinates to screen coordinates
        let transform_x = |x: f64| {
            x as f32 * curve_x_dim + curve_rext.min.x
        };
        let transform_y = |y: f64| {
            y as f32 * - curve_y_dim + curve_rext.max.y
        };
        let transform = |(x, y): (f64, f64)| {
            Pos2::new(
                transform_x(x),
                transform_y(y)
            )
        };
        //transforms from screen coordinates to curve coordinates
        let transform_inv_x = |x: f32| {
            (x as f64 - curve_rext.min.x as f64) / curve_x_dim as f64
        };
        let transform_inv_y = |y: f32| {
            (curve_rext.max.y as f64 - y as f64) / curve_y_dim as f64
        };
        let transform_inv = |p: Pos2| {
            (transform_inv_x(p.x), transform_inv_y(p.y))
        };

        if response.drag_started() && let EditState::PreMoving(point) = self.edit_state {
            self.edit_state = EditState::Moving(point);
        }

        // draw editing edges and points
        if let EditState::Moving(point_id) = self.edit_state {
            let surrounding_points = (
                self.curve.prev_point(point_id),
                self.curve.next_point(point_id)
            );

            // minimum and maximum y coordinates (in ui coords)
            let min_y = curve_rext.min.y;
            let max_y = curve_rext.max.y;

            match surrounding_points {
                (Some(l_point_id), Some(r_point_id)) => {
                    // minimum and maximum coordinates (in ui coords)
                    let min_x = transform_x(self.curve.get_point_time(l_point_id));
                    let max_x = transform_x(self.curve.get_point_time(r_point_id));

                    let l_point = transform(self.curve.get_point_coords(l_point_id));
                    let r_point = transform(self.curve.get_point_coords(r_point_id));
                    let point = Pos2::new(
                        mouse_pos.x.clamp(min_x, max_x),
                        mouse_pos.y.clamp(min_y, max_y)
                    );

                    painter.line_segment([l_point, point], egui::Stroke::new(4.0, Self::POINT_COLOR));
                    painter.line_segment([point, r_point], egui::Stroke::new(4.0, Self::POINT_COLOR));
                    painter.circle_filled(
                        point,
                        Self::POINT_RADIUS,
                        Self::POINT_COLOR
                    );
                }

                (Some(l_point_id), None) => {
                    let l_point = transform(self.curve.get_point_coords(l_point_id));
                    let point = Pos2::new(
                        curve_rext.max.x,
                        mouse_pos.y.clamp(min_y, max_y)
                    );

                    painter.line_segment([l_point, point], egui::Stroke::new(4.0, Self::POINT_COLOR));
                    painter.circle_filled(
                        point,
                        Self::POINT_RADIUS,
                        Self::POINT_COLOR
                    );
                }

                (None, Some(r_point_id)) => {
                    let r_point = transform(self.curve.get_point_coords(r_point_id));
                    let point = Pos2::new(
                        curve_rext.min.x,
                        mouse_pos.y.clamp(min_y, max_y)
                    );

                    painter.line_segment([point, r_point], egui::Stroke::new(4.0, Self::POINT_COLOR));
                    painter.circle_filled(
                        point,
                        Self::POINT_RADIUS,
                        Self::POINT_COLOR
                    );
                }

                (None, None) => {
                    unreachable!("we guarantee that there is at least one segment in the curve");
                }
            }
        }

        // draw non-moving edges
        for (p1_id, p2_id) in self.curve.point_pairs_iter() {
            if self.edit_state.is_moving_point(p1_id) || self.edit_state.is_moving_point(p2_id) {
                continue;
            }

            let p1 = self.curve.get_point_coords(p1_id);
            let p2 = self.curve.get_point_coords(p2_id);

            let point1 = transform(p1);
            let point2 = transform(p2);

            painter.line_segment([point1, point2], egui::Stroke::new(4.0, Self::POINT_COLOR));
        }

        // draw non-moving points
        for point_id in self.curve.point_iter() {
            if !self.edit_state.is_moving_point(point_id) {
                let coords = transform(self.curve.get_point_coords(point_id));

                if let Some(response_pos) = response.interact_pointer_pos() {
                    let on_point = (response_pos - coords).length() <= Self::POINT_RADIUS;
                    if on_point {
                        if response.secondary_clicked() {
                            self.edit_state = EditState::Configuring(
                                point_id, 
                                PointConfigMenu {
                                    time_text: self.curve.get_point_time(point_id).to_string(),
                                    value_text: self.curve.get_point_value(point_id).to_string()
                                }
                            )
                        } else if response.is_pointer_button_down_on() && !self.edit_state.is_moving() {
                            self.edit_state = EditState::PreMoving(point_id);
                        }
                    }
                }

                painter.circle_filled(
                    coords,
                    Self::POINT_RADIUS,
                    if let EditState::Configuring(cfg_id, _) = self.edit_state
                    && self.curve.does_point_contain_partial(point_id, cfg_id) {
                        Self::FOCUS_POINT_COLOR
                    } else {
                        Self::POINT_COLOR
                    }
                );
            }
        }


        // detect if moving has stopped
        if let EditState::Moving(point) = self.edit_state && ui.input(|input| !input.pointer.primary_down()) {
            let y = transform_inv_y(mouse_pos.y).clamp(0.0, 1.0);

            let new_point = if self.curve.point_is_intermediate(point) {
                let x = transform_inv_x(mouse_pos.x);
                self.curve.set_point_time(point, x)
            } else {
                point
            };

            self.curve.set_point_value(new_point, y as f64);
            self.edit_state = EditState::Viewing;
        }

        // detect if editing has stopped
        if response.clicked() && !self.edit_state.is_premoving() {
            self.edit_state = EditState::Viewing;
        }

        // whether or not we should return to view state
        let mut start_viewing = false;

        // draw point config menu
        if let EditState::Configuring(point, menu_data) = &mut self.edit_state {
            let coords = transform(self.curve.get_point_coords(*point));

            let popup_pos = Pos2 {
                x: (coords.x + Self::CONFIG_X_OFFSET).clamp(
                    curve_rext.min.x + Self::POPUP_PADDING,
                    curve_rext.max.x - Self::CONFIG_WIDTH - Self::POPUP_PADDING
                ),
                y: (coords.y + Self::CONFIG_Y_OFFSET).clamp(
                    curve_rext.min.y + Self::POPUP_PADDING,
                    curve_rext.max.y - Self::CONFIG_HEIGHT - Self::POPUP_PADDING
                ),
            };

            let popup_rect = Rect::from_min_size(popup_pos, Vec2::new(Self::CONFIG_WIDTH, Self::CONFIG_HEIGHT));

            ui.scope_builder(UiBuilder::new().max_rect(popup_rect), |mut ui| {
                let frame = Frame::new();
                frame.show(&mut ui, |ui| {
                    ui.label("Value:");
                    let mut value = coords.y;
                    if utils::non_neg_number_input(ui, &mut menu_data.value_text, &mut value) {
                        self.curve.set_point_value(*point, value as f64);
                        menu_data.value_text = self.curve.get_point_value(*point).to_string();
                    }

                    if self.curve.point_is_intermediate(*point) {
                        ui.label("Time:");
                        let mut time = coords.x;
                        if utils::non_neg_number_input(ui, &mut menu_data.time_text, &mut time) {
                            *point = self.curve.set_point_time(*point, time as f64);
                            menu_data.time_text = self.curve.get_point_time(*point).to_string();
                        }

                        if ui.button("Delete Point").clicked() {
                            self.curve.remove_point(*point);
                            start_viewing = true;
                        }
                    }

                })
            });
        }
        
        if start_viewing {
            self.edit_state = EditState::Viewing
        }

    }
}

impl eframe::App for CurveEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("test")
            .resizable(true)
            .show(ctx, |ui| self.draw(ui) );
    }
}

