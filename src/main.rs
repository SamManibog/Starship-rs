use starship_rust::{
    circuit::CircuitBuilderSpecification as Cbs,
    circuits::{ConstantBuilder, InterpolatorBuilder, OscillatorBuilder, RouterBuilder, SampleQuantizerBuilder, SwitchBuilder},
};

macro_rules! builder_defs {
    ($({$t:ty : $n:expr})*) => (
        [
            $(Cbs::new($n, || Box::new(<$t>::new())),)*
        ]
    )
}

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };

    let builders = builder_defs![
        {ConstantBuilder: "Constant"}
        {InterpolatorBuilder: "Interpolator"}
        {RouterBuilder: "Router"}
        {OscillatorBuilder: "Oscillator"}
        {SwitchBuilder: "Switch"}
        {SampleQuantizerBuilder: "S-Quantizer"}
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
