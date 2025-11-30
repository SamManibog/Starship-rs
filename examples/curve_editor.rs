use egui::{Pos2, Rect, Sense, Vec2};
use starship_rust::sequencers::curve::{Curve, CurvePointId};

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

struct CurveEditor {
    curve: Curve,
    editing_point: Option<CurvePointId>,
    saved_mouse_pos: Pos2,
}

impl CurveEditor {
    pub const POINT_RADIUS: f32 = 6.0;
    pub const POINT_COLOR: egui::Color32 = egui::Color32::WHITE;

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
            editing_point: None,
            saved_mouse_pos: Pos2::ZERO
        }
    }
}

impl eframe::App for CurveEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let request_dim = ui.max_rect().max - ui.max_rect().min;
            let (response, painter) = ui.allocate_painter(request_dim, egui::Sense::click_and_drag());
            let main_rect = response.rect;
            let x_dim = main_rect.max.x - main_rect.min.x;
            let y_dim = main_rect.max.y - main_rect.min.y;

            // mouse/interaction position relative to current ui
            let mouse_pos =  ui.input(|input| {
                if let Some(pos) = input.pointer.latest_pos() {
                    self.saved_mouse_pos = pos;
                    pos
                } else {
                    self.saved_mouse_pos
                }
            });

            let transform_x = |x: f64| {
                x as f32 * x_dim + main_rect.min.x
            };
            let transform_y = |y: f64| {
                y as f32 * y_dim + main_rect.min.y
            };
            let transform = |(x, y): (f64, f64)| {
                Pos2::new(
                    transform_x(x),
                    transform_y(y)
                )
            };

            // draw editing edges and points
            if let Some(point_id) = self.editing_point {
                let surrounding_points = (
                    self.curve.prev_point(point_id),
                    self.curve.next_point(point_id)
                );

                // minimum and maximum y coordinates (in ui coords)
                let min_y = main_rect.min.y;
                let max_y = main_rect.max.y;

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
                            main_rect.max.x,
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
                            main_rect.min.x,
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

            // draw non-editing edges
            for (p1_id, p2_id) in self.curve.point_pairs_iter() {
                if Some(p1_id) == self.editing_point || Some(p2_id) == self.editing_point {
                    continue;
                }

                let p1 = self.curve.get_point_coords(p1_id);
                let p2 = self.curve.get_point_coords(p2_id);

                let point1 = transform(p1);
                let point2 = transform(p2);

                painter.line_segment([point1, point2], egui::Stroke::new(4.0, Self::POINT_COLOR));
            }

            // draw non-editing points
            let point_response_size = Vec2::new(Self::POINT_RADIUS * 2.0, Self::POINT_RADIUS * 2.0);
            for point_id in self.curve.point_iter() {
                if Some(point_id) != self.editing_point {
                    let coords = transform(self.curve.get_point_coords(point_id));

                    let point_response = ui.allocate_rect(
                        Rect::from_center_size(coords, point_response_size),
                        Sense::drag()
                    );
                    
                    if point_response.drag_started() {
                        self.editing_point = Some(point_id);
                    }
                    painter.circle_filled(
                        coords,
                        Self::POINT_RADIUS,
                        Self::POINT_COLOR
                    );
                }
            }

            if ui.input(|input| !input.pointer.primary_down()) {
                if let Some(point) = self.editing_point {
                    let y = (mouse_pos.y - main_rect.min.y) / y_dim;

                    self.curve.set_point_value(point, y as f64);

                    if self.curve.point_is_intermediate(point) {
                        let min_x = self.curve.get_point_time(
                            self.curve.prev_point(point).unwrap()
                        );
                        let max_x = self.curve.get_point_time(
                            self.curve.next_point(point).unwrap()
                        );
                        let x_raw = ((mouse_pos.x - main_rect.min.x) / x_dim) as f64;
                        self.curve.set_point_time(point, x_raw.clamp(min_x, max_x));
                    }
                }
                self.editing_point = None
            }

        });
    }
}

