use crate::circuit::{BuildState, Circuit, CircuitBuilder, CircuitSpecification};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SampleQuantizerKind {
    Multiple,
    MajorScale,
    Semitone,
    Microtone
}

impl SampleQuantizerKind {
    const MULTPILE_TEXT: &'static str = "Multiple S-Quantizer";
    const MAJOR_TEXT: &'static str = "Major S-Quantizer";
    const SEMITONE_TEXT: &'static str = "Semitone S-Quantizer";
    const MICROTONE_TEXT: &'static str = "Microtone S-Quantizer";

    fn display_string(&self) -> &'static str {
        match self {
            Self::Multiple => Self::MULTPILE_TEXT,
            Self::MajorScale => Self::MAJOR_TEXT,
            Self::Semitone => Self::SEMITONE_TEXT,
            Self::Microtone => Self::MICROTONE_TEXT,
        }
    }
}

impl std::fmt::Display for SampleQuantizerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.display_string()
        )
    }
}

#[derive(Debug, Clone)]
pub struct SampleQuantizerBuilder {
    kind: SampleQuantizerKind
}

impl SampleQuantizerBuilder {
    const SPECIFICATION: CircuitSpecification = CircuitSpecification {
        input_names: &["Sample", "Fundemental"],
        output_names: &["Out"],
        size: egui::vec2(200.0, 200.0),
        playback_size: None,
    };

    pub fn new() -> Self {
        Self{
            kind: SampleQuantizerKind::Multiple
        }
    }
}

impl CircuitBuilder for SampleQuantizerBuilder {
    fn show(&mut self, ui: &mut egui::Ui) {
        ui.radio_value(&mut self.kind, SampleQuantizerKind::Multiple, SampleQuantizerKind::MULTPILE_TEXT);
        ui.radio_value(&mut self.kind, SampleQuantizerKind::MajorScale, SampleQuantizerKind::MAJOR_TEXT);
        ui.radio_value(&mut self.kind, SampleQuantizerKind::Semitone, SampleQuantizerKind::SEMITONE_TEXT);
        ui.radio_value(&mut self.kind, SampleQuantizerKind::Microtone, SampleQuantizerKind::MICROTONE_TEXT);
    }

    fn name(&self) -> &str {
        self.kind.display_string()
    }

    fn specification(&self) -> &'static CircuitSpecification {
        &Self::SPECIFICATION
    }

    fn build(&self, _: &BuildState) -> Box<dyn Circuit> {
        match self.kind {
            SampleQuantizerKind::Multiple => Box::new(MultipleSampleQuantizer{}),
            SampleQuantizerKind::MajorScale => Box::new(EtMajorSampleQuantizer{}),
            SampleQuantizerKind::Semitone => Box::new(EtSemitoneSampleQuantizer{}),
            SampleQuantizerKind::Microtone => Box::new(EtMicrotoneSampleQuantizer{}),
        }
    }
}

/// Quantizes the given sample to the nearest multiple of the given fundamental
#[derive(Debug, Default)]
pub struct MultipleSampleQuantizer {}

impl Circuit for MultipleSampleQuantizer {
    fn operate(&mut self, inputs: &[f32], outputs: &mut[f32], _: f32) {
        let sample = inputs[0];
        let fundamental = inputs[1];
        outputs[0] = (sample / fundamental).round() * fundamental;
    }
}

/// Quantizes the given sample to the nearest major scale note with the given root
#[derive(Debug, Default)]
pub struct EtMajorSampleQuantizer {}

impl Circuit for EtMajorSampleQuantizer {
    fn operate(&mut self, inputs: &[f32], outputs: &mut[f32], _: f32) {
        let sample = inputs[0];
        let root = inputs[1];
        outputs[0] = crate::pitch::equal_temperment::quantize_major_scale(root, sample);
    }
}

/// Quantizes the given sample to the nearest semitone with the given value of A4
#[derive(Debug, Default)]
pub struct EtSemitoneSampleQuantizer {}

impl Circuit for EtSemitoneSampleQuantizer {
    fn operate(&mut self, inputs: &[f32], outputs: &mut[f32], _: f32) {
        let sample = inputs[0];
        let a4 = inputs[1];
        outputs[0] = crate::pitch::equal_temperment::quantize_semitone(a4, sample);
    }
}

/// Quantizes the given sample to the nearest semitone with the given value of A4
#[derive(Debug, Default)]
pub struct EtMicrotoneSampleQuantizer {}

impl Circuit for EtMicrotoneSampleQuantizer {
    fn operate(&mut self, inputs: &[f32], outputs: &mut[f32], _: f32) {
        let sample = inputs[0];
        let a4 = inputs[1];
        outputs[0] = crate::pitch::equal_temperment::quantize_microtone(a4, sample);
    }
}

