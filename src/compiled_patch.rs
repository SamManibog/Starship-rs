use std::{collections::{HashMap, HashSet}, time::Duration};

use cpal::{traits::DeviceTrait, BuildStreamError, FromSample, OutputCallbackInfo, Sample, SampleFormat, SampleRate, StreamConfig, StreamError};

use crate::{
    circuit::{BuildState, Circuit, CircuitBuilder, CircuitUiSlot}, circuit_id::{CircuitId, CircuitPortId, PortId, PortKind}, connection_manager::ConnectionManager, pitch::TuningSystem
};

/// The intermediate representation of a patch, just before total compilation
pub struct PatchIr<'a> {
    /// A list of circuit ids in the order they will operate
    circuits: Vec<CircuitId>,

    builders: &'a HashMap<CircuitId, Box<dyn CircuitBuilder>>,

    connections: &'a ConnectionManager,

    /// The range of indices that each circuit takes input from, exclusive
    circuit_input_ranges: Vec<(usize, usize)>,

    /// otl[circuit_index][outgoing_port_index][i] = destination_port_index
    circuit_target_list: Vec<Vec<Vec<ConnectionBehavior>>>,

    /// itl[input_index][i] = destination_port_index
    input_target_lists: Vec<Vec<usize>>,

    output_count: usize,
}

impl<'a> PatchIr<'a> {
    pub fn new(
        ids: &[CircuitId],
        builders: &'a HashMap<CircuitId, Box<dyn CircuitBuilder>>,
        connections: &'a ConnectionManager,
        inputs: &[HashSet<CircuitId>],
        outputs: &[HashSet<CircuitId>],
    ) -> Self {
        // determine circuit order and depths (excludes inputs)
        let (circuits, depths) = Self::compute_order(
            builders,
            connections,
            inputs,
            ids.iter()
                .filter_map(|id| {
                    let mut out = None;
                    for set in outputs {
                        if set.contains(id) {
                            out = Some(*id);
                            break;
                        }
                    }
                    out
                })
                .collect()
        );

        // determine send/save behavior

        let (
            // a map from a CircuitId to its index in self::circuits
            circuit_id_to_index_map,

            // a map from a CircuitPortId to its index in self::input_buffer
            port_id_to_index_map,

            // a vector where each item at index i corresponds to the range of indices
            // read by the circuit self::circuits[i] for input. These ranges are exclusive.
            input_ranges
        ) = Self::construct_index_maps_and_ranges(&circuits, builders);

        // the index in self::input_buffer at and after which output circuits exist
        let out_start_index = if let Some((_, end)) = input_ranges.last() {
            *end
        } else {
            0
        };

        // otl[circuit_index][outgoing_port_index][i] = destination_port_index
        let mut output_target_list = vec![];

        // itl[input_index][i] = destination_port_index
        let mut input_target_lists = vec![];

        // add space for input's targets in itl
        for _ in inputs {
            input_target_lists.push(Vec::new());
        }

        // Iterate over all circuits
        // (the index of the circuit being handled, the id of the circuit)
        for (circuit_index, circuit) in circuits.iter().enumerate() {
            // handle case where we have a special input
            if let Some(input_index) = Self::hashset_array_contains(inputs, circuit) {
                let output_targets = &mut input_target_lists[input_index];

                let out_port = CircuitPortId::new(*circuit, PortId::new(0, PortKind::Output));

                // the list of targets (circuit_port_ids) for each port
                if let Some(destinations) = connections.port_query_ports(out_port) {

                    // Iterate over all output targets
                    // Determine processing behavior for each target
                    for dst in destinations {
                        // the circuit id belonging to the destination port
                        let dst_circuit = dst.circuit_id;
                        if let Some(index) = Self::hashset_array_contains(outputs, &dst_circuit) {
                            output_targets.push(out_start_index + index);

                            // the index of the destination circuit
                            // if the circuit is not in our map, we know it is not
                            // necessary for *audio* processing.
                        } else if circuit_id_to_index_map.contains_key(&dst_circuit)  {
                            output_targets.push(port_id_to_index_map[&dst]);
                        }
                    }
                }

            } else {
                //handle case where we are not adding a special input
                // opt[outgoing_port_index][i] = destination_port_index
                let mut output_ports = vec![];

                // the depth the the circuit
                let circuit_depth = depths[circuit_index];

                // the specification of the circuit
                let spec = builders[circuit].specification();


                // Iterate over all output ports
                for out_port in spec.circuit_output_port_id_iter(*circuit) {
                    // ot[i] = destination_port_index
                    let mut output_targets = vec![];

                    // the list of targets (circuit_port_ids) for each port
                    if let Some(destinations) = connections.port_query_ports(out_port) {

                        // Iterate over all output targets
                        // Determine processing behavior for each target
                        for dst in destinations {
                            // the circuit id belonging to the destination port
                            let dst_circuit = dst.circuit_id;


                            if let Some(index) = Self::hashset_array_contains(outputs, &dst_circuit) {
                                // handle case when we are sending to an output circuit
                                output_targets.push(ConnectionBehavior::new(
                                    Behavior::Send,
                                    out_start_index + index
                                ));

                                // the index of the destination circuit
                                // if the circuit is not in our map, we know it is not
                                // necessary for *audio* processing.
                            } else if let Some(dst_circuit_index) = circuit_id_to_index_map.get(&dst_circuit)  {
                                // handle case when we are sending to a non-output circuit

                                // the behavior to use when handling output
                                let behavior = if depths[*dst_circuit_index] < circuit_depth {
                                    Behavior::Send
                                } else {
                                    Behavior::Save
                                };
                                output_targets.push(ConnectionBehavior::new(
                                    behavior,
                                    port_id_to_index_map[&dst]
                                ));
                            }
                        }
                    }
                    output_ports.push(output_targets);
                }
                output_target_list.push(output_ports);
            }
        }

        Self {
            circuits,
            builders,
            input_target_lists,
            connections,
            circuit_input_ranges: input_ranges,
            circuit_target_list: output_target_list,
            output_count: outputs.len(),
        }
    }

    /// Given a list of CircuitIds and their builders, constructs a map from the id
    /// of the circuit to its position in the passed CircuitId list, constructs a
    /// map from the ids of each port in the list of circuits, to its index in a
    /// proposed input buffer, described by the third construction: a vector specifying
    /// which indices on the input buffer belong to which circuit.
    ///
    /// Assumptions:
    ///  - Circuits does not contain any outputs or inputs
    ///  - builders.keys is a superset of circuits.elements
    ///
    fn construct_index_maps_and_ranges(
        circuits: &[CircuitId],
        builders: &HashMap<CircuitId, Box<dyn CircuitBuilder>>,
    ) -> (HashMap<CircuitId, usize>, HashMap<CircuitPortId, usize>, Vec<(usize, usize)>) {
        // a map from a CircuitId to its corresponding index in Self::circuits
        let mut circuit_id_to_index_map: HashMap<CircuitId, usize> = HashMap::new();

        // a map from a CircuitPortId to its corresponding index in Self::input_buffer
        let mut port_id_to_index_map: HashMap<CircuitPortId, usize> = HashMap::new();

        // the value to be used for Self::input_ranges
        let mut input_ranges: Vec<(usize, usize)> = vec![];

        let mut port_index = 0;

        // fill input ranges with "normal circuits"
        for (circuit_index, circuit) in circuits.iter().enumerate() {
            circuit_id_to_index_map.insert(*circuit, circuit_index);

            // handle normal circuits
            // the index this circuit's range begins on
            let start_index = port_index;
            let spec = builders[circuit].specification();
            for input_port in spec.circuit_input_port_id_iter(*circuit) {
                port_id_to_index_map.insert(input_port, port_index);
                port_index += 1;
            }
            input_ranges.push((start_index, port_index));
        }

        (
            circuit_id_to_index_map,
            port_id_to_index_map,
            input_ranges
        )
    }

    /// Determines the index of the hashset in the given array that the id belongs to
    /// Assumes keys stored in the hashset are disjoint
    fn hashset_array_contains(set_array: &[HashSet<CircuitId>], id: &CircuitId) -> Option<usize> {
        for (i, set) in set_array.iter().enumerate() {
            if set.contains(id) {
                return Some(i);
            }
        }
        None
    }

    /// Determine the order and depth of the circuits using a breadth first search.
    /// BFS starts at the speakers. The function then returns two lists where the data at
    /// a given index are associated. The first list is a list of CircuitIds, the second list
    /// is the depth of the corresponding id. Both lists are provided in order of greatest
    /// to least depth.
    /// builders - A map from a circuit id to its respective builder
    /// connections - A connection manager detailing the connections between builders
    /// outputs - a vector containing all circuit ids corresponding to a special output
    ///
    /// Assumes ids in outputs correspond to circuits that only have input ports
    ///
    /// Order vector does not contain any output or input circuits
    fn compute_order(
        builders: &HashMap<CircuitId, Box<dyn CircuitBuilder>>,
        connections: &ConnectionManager,
        inputs: &[HashSet<CircuitId>],
        outputs: Vec<CircuitId>
    ) -> (Vec<CircuitId>, Vec<usize>) {
        // the reversed order at which to update circuits
        let mut circuits_rev: Vec<CircuitId> = vec![];

        // the distance from each circuit to a speaker
        let mut depths_rev: Vec<usize> = vec![];

        // a list of all circuits already visited or queued
        let mut visited: HashSet<CircuitId> = HashSet::new();

        // the current list of circuits to search
        // circuits in this queue all have the same depth
        let mut current_queue = outputs;

        // the next list of circuits to search
        // circuits in this queue all have the same depth
        let mut next_queue = vec![];

        // the current depth of circuit
        let mut depth = 0;

        while !current_queue.is_empty() {
            // the current circuit being traversed
            let current = current_queue.pop().unwrap();

            // the specification of the circuit being traversed
            // used to get an iterator over all input ports
            let specification = builders[&current].specification();

            'inner: for input_port in specification.circuit_input_port_id_iter(current) {
                let source_ports = connections.port_query_ports(input_port);
                if source_ports == None {
                    continue 'inner;
                }

                for source_port in source_ports.unwrap() {
                    let source_circuit = source_port.circuit_id;

                    if !visited.contains(&source_circuit) && Self::hashset_array_contains(&inputs, &source_circuit) == None {
                        visited.insert(source_circuit);
                        circuits_rev.push(source_circuit);
                        next_queue.push(source_circuit);
                        depths_rev.push(depth);
                    }
                }
            }

            // rotate queues after the current queue is deleted
            // if there is nothing in next_queue, the loop will terminate
            if current_queue.is_empty() {
                current_queue = next_queue;
                next_queue = vec![];
            }

            //incriment depth
            depth += 1;
        }

        circuits_rev.reverse();
        depths_rev.reverse();

        debug_assert!(circuits_rev.len() == depths_rev.len(), "Circuits and depths should be equal");

        (circuits_rev, depths_rev)
    }

    /// Constructs self as well as the associated ui slots
    pub fn compile(
        &self,
        sample_rate: u32,
        sample_multiplier: f32,
    ) -> CompiledPatch {
        // initialize the input buffer
        let input_buffer = vec![0.0; self.circuits.len() + self.output_count];

        // todo TEMPORARY PLEASE DELETE
        let tuning = TuningSystem::EqualTemperment(440.0);

        let mut built_circuits = Vec::with_capacity(self.circuits.len());
        let mut ui_slots = Vec::new();

        for circuit_id in &self.circuits {
            let builder = &self.builders[&circuit_id];
            let specification = builder.specification();

            // construct build state
            let input_counts: Vec<usize> = specification.circuit_input_port_id_iter(*circuit_id)
                .filter_map(|id| self.connections.port_query_connection_count(id))
                .collect();
            let output_counts: Vec<usize> = specification
                .circuit_output_port_id_iter(*circuit_id)
                .filter_map(|id| self.connections.port_query_connection_count(id))
                .collect();
            let expect_ui = specification.playback_size != None;

            let mut build_state = BuildState::new(
                &input_counts,
                &output_counts,
                tuning,
                sample_rate,
                expect_ui
            );

            // build
            built_circuits.push(builder.build(&build_state));

            if expect_ui {
                ui_slots.push(CircuitUiSlot {
                    size: specification.playback_size.unwrap(),
                    ui: build_state.get_ui()
                })
            }
        }

        /*
        debug_assert!(
        built_circuits.len() == self.output_target_list.len(),
        "built_circuits should have as many elements as output_target_list. built_circuits: {}; otl: {}",
        built_circuits.len(),
        self.output_target_list.len()
        );

        debug_assert!(
        built_circuits.len() == input_ranges.len(),
        "built_circuits should have as many elements input_ranges. built_circuits: {}; input_ranges: {}",
        built_circuits.len(),
        input_ranges.len()
        );
        */

        CompiledPatch {
            circuits: built_circuits,
            circuit_input_buffer: input_buffer,
            circuit_input_ranges: self.circuit_input_ranges.clone(),
            circuit_target_list: self.circuit_target_list.clone(),
            input_target_lists: self.input_target_lists.clone(),
            sample_multiplier,
            input_count: self.input_target_lists.len(),
            output_count: self.output_count,
        }
    }
}

pub struct CompiledPatch {
    /// The list of circuits used in processing. In order.
    circuits: Vec<Box<dyn Circuit>>,

    /// The buffer that circuits read from
    circuit_input_buffer: Vec<f32>,

    /// The range of indices that each circuit takes input from, exclusive
    circuit_input_ranges: Vec<(usize, usize)>,

    /// otl[circuit_index][outgoing_port_index][i] = destination_port_index
    circuit_target_list: Vec<Vec<Vec<ConnectionBehavior>>>,

    /// itl[input_index][i] = destination_port_index
    input_target_lists: Vec<Vec<usize>>,

    /// the value to multiply all samples by
    sample_multiplier: f32,

    /// the number of inputs this patch takes
    pub input_count: usize,

    /// the number of outputs this patch takes
    pub output_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Behavior {
    Send,
    Save
}

#[derive(Clone, Copy)]
struct ConnectionBehavior {
    /// First bit specifies behavior, rest of bits specify index
    /// First bit is 1 -> Save
    /// First bit is 0 -> Send
    data: usize
}

impl ConnectionBehavior {
    pub fn new(behavior: Behavior, index: usize) -> Self {
        let data = if behavior == Behavior::Send {
            index
        } else {
            ((index << 1) + 1).rotate_right(1)
        };

        Self {
            data
        }
    }

    pub fn behavior(&self) -> Behavior {
        if self.data.leading_zeros() > 0 {
            Behavior::Send
        } else {
            Behavior::Save
        }
    }

    pub fn index(&self) -> usize {
        (self.data << 1) >> 1
    }
}

impl std::fmt::Debug for ConnectionBehavior {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "behavior: {:?}, index: {:?}, raw: {:?}",
            self.behavior(),
            self.index(),
            self.data
        )
    }
}

impl CompiledPatch {
    /// Updates all circuits once and in order for one sample
    /// Returns the value of the sample as an f32
    pub fn update(&mut self, inputs: &[f32], output: &mut [f32], delta: f32) {
        debug_assert!(inputs.len() == self.input_count, "Input array size must match input count.");
        debug_assert!(output.len() == self.output_count, "Output array size must match output count.");

        // send inputs to circuits that depend on them
        for i in 0..inputs.len() {
            let value = inputs[i];
            let input_target_list = &mut self.input_target_lists[i];
            for target in input_target_list {
                self.circuit_input_buffer[*target] += value;
            }
        }

        // the buffer where save-behavior items are stored
        let mut save_buffer = vec![0.0; self.circuit_input_buffer.len()];

        // handle internal updates
        for i in self.input_count..self.circuits.len() {
            // the current circuit to update
            let circuit = &mut self.circuits[i];

            // the range of inputs associated with the circuit
            let range = self.circuit_input_ranges[i];

            // the slice the circuit should take input from
            let inputs = &self.circuit_input_buffer[range.0..range.1];

            // the buffer the circuit should write to
            let mut output_buffer = vec![0.0; self.circuit_target_list[i].len()];

            circuit.operate(&inputs, &mut output_buffer, delta);

            // iterate through each output port to send or save the result
            for j in 0..output_buffer.len() {
                // the value stored at the current output slot
                let output_value = output_buffer[j];

                // the list of targets that the output should be sent to
                let targets = &self.circuit_target_list[i][j];

                // iterate through each output target to send or save the result
                for target in targets {
                    match target.behavior() {
                        Behavior::Send => {
                            self.circuit_input_buffer[target.index()] += output_value;
                        }
                        Behavior::Save => {
                            save_buffer[target.index()] += output_value;
                        }
                    }
                }
            }
        }

        // send output
        for i in (self.circuit_input_buffer.len() - self.input_count)..self.circuit_input_buffer.len() {
            output[i] = self.circuit_input_buffer[i];
        }

        // swap buffers
        self.circuit_input_buffer = save_buffer;
    }

    /*
    pub fn get_sample<T: Sample + FromSample<f32>>(
        &mut self,
        delta: f32
    ) -> T {
        Sample::to_sample::<T>(self.update(delta) * self.sample_multiplier)
    }

    /// Converts the backend to a callback used for an audio stream
    pub fn stream_data_callback<T: Sample + FromSample<f32>>(
        mut self,
        sample_rate: SampleRate
    ) -> impl FnMut(&mut [T], &OutputCallbackInfo) {
        let delta = (1.0_f64 / (sample_rate.0 as f64)) as f32;
        move |data, _callback_info| {
            for sample in data.iter_mut() {
                *sample = self.get_sample(delta);
            }
        }
    }

    /// Creates an output stream, consuming self
    pub fn into_output_stream<D: DeviceTrait, E:FnMut(StreamError) + Send + 'static>(
        self,
        device: &D,
        config: &StreamConfig,
        error_callback: E,
        timeout: Option<Duration>,
        sample_format: SampleFormat,
        sample_rate: SampleRate
    ) -> Result<D::Stream, BuildStreamError> {
        match sample_format {
            SampleFormat::I8 => {
                device.build_output_stream(
                    &config,
                    self.stream_data_callback::<i8>(sample_rate),
                    error_callback,
                    timeout
                )
            },
            SampleFormat::I16 => {
                device.build_output_stream(
                    config,
                    self.stream_data_callback::<i16>(sample_rate),
                    error_callback,
                    timeout
                )
            },
            SampleFormat::I32 => {
                device.build_output_stream(
                    &config,
                    self.stream_data_callback::<i32>(sample_rate),
                    error_callback,
                    timeout
                )
            },
            SampleFormat::I64 => {
                device.build_output_stream(
                    &config,
                    self.stream_data_callback::<i64>(sample_rate),
                    error_callback,
                    timeout
                )
            },
            SampleFormat::U8 => {
                device.build_output_stream(
                    &config,
                    self.stream_data_callback::<u8>(sample_rate),
                    error_callback,
                    timeout
                )
            },
            SampleFormat::U16 => {
                device.build_output_stream(
                    &config,
                    self.stream_data_callback::<u16>(sample_rate),
                    error_callback,
                    timeout
                )
            },
            SampleFormat::U32 => {
                device.build_output_stream(
                    &config,
                    self.stream_data_callback::<u32>(sample_rate),
                    error_callback,
                    timeout
                )
            },
            SampleFormat::U64 => {
                device.build_output_stream(
                    &config,
                    self.stream_data_callback::<u64>(sample_rate),
                    error_callback,
                    timeout
                )
            },
            SampleFormat::F32 => {
                device.build_output_stream(
                    &config,
                    self.stream_data_callback::<f32>(sample_rate),
                    error_callback,
                    timeout
                )
            },
            SampleFormat::F64 => {
                device.build_output_stream(
                    &config,
                    self.stream_data_callback::<f64>(sample_rate),
                    error_callback,
                    timeout
                )
            },
            _ => panic!("Unsupported stream format.")
        }
    }
    */

}

