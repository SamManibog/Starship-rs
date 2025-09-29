use starship_rust::{
    circuit::CircuitBuilderSpecification as Cbs,
    circuits::{ConstantBuilder, OscillatorBuilder, RouterBuilder, SwitchBuilder},
};

fn main() -> eframe::Result {

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };

    let builders = [
        Cbs::new("Constant", || Box::new(ConstantBuilder::new())),
        Cbs::new("Router", || Box::new(RouterBuilder::new())),
        Cbs::new("Oscillator", || Box::new(OscillatorBuilder::new())),
        Cbs::new("Switch", || Box::new(SwitchBuilder::new())),
    ];

    eframe::run_native(
        "Starship",
        native_options,
        Box::new(|cc| {
            Ok(Box::new(
                starship_rust::app::App::new(cc, &builders)
            ))
        })
    )
}
