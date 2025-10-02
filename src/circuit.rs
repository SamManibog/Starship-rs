use std::cell::{Cell, OnceCell};

use egui::{Label, Ui, Vec2};

use crate::{circuit_id::{CircuitId, CircuitPortId, PortId, PortKind}, pitch::TuningSystem};

/// The specification "skeleton" for a circuit. Describes basic top-level capabilities of
/// the circuit.
#[derive(Debug)]
pub struct CircuitSpecification {
    /// The names of each input to the circuit.
    pub input_names: &'static[&'static str],

    /// The names of each output of the circuit
    pub output_names: &'static[&'static str],

    /// The size of the circuit in the editor
    pub size: Vec2,

    /// The size of the frontend ui for the circuit during playback
    /// Should be none if there is no used ui.
    pub playback_size: Option<Vec2>
}

impl CircuitSpecification {
    /// Returns an iterator over input port ids
    pub fn input_port_id_iter(&self) -> impl Iterator<Item = PortId> {
        (0..self.input_names.len())
        	.into_iter()
            .map(|index| PortId::new(index, PortKind::Input))
    }

    /// Returns an iterator over output port ids
    pub fn output_port_id_iter(&self) -> impl Iterator<Item = PortId> {
        (0..self.output_names.len())
        	.into_iter()
            .map(|index| PortId::new(index, PortKind::Output))
    }

    /// Returns an iterator over all port ids
    pub fn port_id_iter(&self) -> impl Iterator<Item = PortId> {
        self.output_port_id_iter().chain(self.input_port_id_iter())
    }

    /// Returns an iterator over all circuit input port ids
    pub fn circuit_input_port_id_iter(&self, circuit: CircuitId) -> impl Iterator<Item = CircuitPortId> {
        self.input_port_id_iter().map::<CircuitPortId, _>(move |id| CircuitPortId::new(circuit, id))
    }

    /// Returns an iterator over all circuit output port ids
    pub fn circuit_output_port_id_iter(&self, circuit: CircuitId) -> impl Iterator<Item = CircuitPortId> {
        self.output_port_id_iter().map::<CircuitPortId, _>(move |id| CircuitPortId::new(circuit, id))
    }

    /// Returns an iterator over all circuit port ids
    pub fn circuit_port_id_iter(&self, circuit: CircuitId) -> impl Iterator<Item = CircuitPortId> {
        self.port_id_iter().map::<CircuitPortId, _>(move |id| CircuitPortId::new(circuit, id))
    }
}


pub struct CircuitBuilderSpecification {
    pub display_name: String,
    pub instance: Box<dyn Fn()->Box<dyn CircuitBuilder>>
}

impl CircuitBuilderSpecification {
    pub fn new(name: &str, instance: impl Fn()->Box<dyn CircuitBuilder> + 'static) -> Self {
        Self {
            display_name: name.into(),
            instance: Box::new(instance)
        }
    }
}

impl std::fmt::Debug for CircuitBuilderSpecification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "display_name: {}", self.display_name)
    }
}

/// Creates a circuit based on user parameters
pub trait CircuitBuilder: std::fmt::Debug {
    /// Draw the circuit UI to the screen. Passed to egui's show function.
    /// Do not attempt to handle circuit connections in this step.
    fn show(&mut self, ui: &mut Ui) {
        ui.add(Label::new("This circuit is not configurable.").wrap());
    }

    /// gets the specification for the circuit
    fn specification(&self) -> &'static CircuitSpecification;

    /// Build the associated circuit and its ui
    fn build(&self, state: &BuildState) -> Box<dyn Circuit>;

    /// gets the name of the circuit being built
    fn name(&self) -> &str;

    /// Request a size for the entire UI.
    /// This size will be filled with the title, IO ports, padding, etc. along with your custom UI.
    /// Called every frame before drawing.
    fn request_size(&self) -> Option<egui::Vec2> { None }
}

/// A circuit that processes signals into outputs
pub trait Circuit: std::fmt::Debug + Send {
    /// Handles a vector of signals to produce some output signals.
    fn operate(&mut self, inputs: &[f32], outputs: &mut[f32], delta: f32);
}

/// The ui for a circuit
pub trait CircuitUi {
    /// Draws the ui to the screen
    fn show(&mut self, ui: &mut Ui);
}

/// Data passed to CircuitBuilders during builds
pub struct BuildState<'a> {
    pub input_counts: &'a [usize],
    pub output_counts: &'a [usize],
    pub tuning: TuningSystem,
    pub sample_rate: u32,
    ui_slot: OnceCell<Box<dyn CircuitUi>>,
    ui_state: Cell<BuildUiState>,
}

impl<'a> BuildState<'a> {
    /// Creates a new build state
    pub fn new(
        input_counts: &'a [usize],
        output_counts: &'a [usize],
        tuning: TuningSystem,
        sample_rate: u32,
        expect_ui: bool
    ) -> Self {
        let ui_state = if expect_ui {
            BuildUiState::Expected
        } else {
            BuildUiState::Disallow
        };

        Self {
            input_counts,
            output_counts,
            tuning,
            sample_rate,
            ui_slot: OnceCell::new(),
            ui_state: Cell::new(ui_state)
        }
    }

    /// Adds a ui to the build state
    pub fn add_ui(&self, ui: Box<dyn CircuitUi>) {
        // debug only as an added value when disallowed is just ignored
        debug_assert!(self.ui_state.get() != BuildUiState::Disallow, "Attempted to add a UI when none were expected.");

        // debug only as the old value is just discarded, but the dev should be made aware of this.
        debug_assert!(self.ui_state.get() != BuildUiState::Recieved, "Attempted to add a UI when one has already been added (only one UI is allowed per circuit).");

        let _ = self.ui_slot.set(ui);
        self.ui_state.set(BuildUiState::Recieved);
    }

    /// Gets the added ui, throws an error in debug mode if a ui was expected, but
    /// none recieved.
    pub(crate) fn get_ui(&mut self) -> Box<dyn CircuitUi> {
        assert!(self.ui_state.get() != BuildUiState::Disallow, "Not expected to recieve a UI; therefore cannot retrieve one.");
        assert!(self.ui_state.get() != BuildUiState::Expected, "Expected to get a ui, but none was recieved.");
        self.ui_slot.take().unwrap()
    }
}

/// Internal data that holds a circuit ui
/// Tracks the size of the ui added
pub struct CircuitUiSlot {
    pub size: Vec2,
    pub ui: Box<dyn CircuitUi>
}

impl CircuitUiSlot {
    pub fn show(&mut self, ui: &mut Ui) {
        self.ui.show(ui);
    }
}

/// enum used to track ui additions during build state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BuildUiState {
    Expected,
    Recieved,
    Disallow,
}
