use std::{collections::{HashMap, HashSet}, time::Duration};

use cpal::{traits::DeviceTrait, BuildStreamError, FromSample, OutputCallbackInfo, Sample, SupportedStreamConfig, SampleFormat, StreamError, StreamConfig};

use crate::{
    circuit::{Circuit, CircuitBuilder}, circuit_id::{CircuitId, CircuitPortId}, connection_manager::ConnectionManager
};

pub struct PlaybackBackendData {
    /// The list of circuits used in processing. In order.
    circuits: Vec<Box<dyn Circuit>>,

    /// The buffer that circuits read from
    input_buffer: Vec<f32>,

    /// The range of indices that each circuit takes input from, exclusive
    input_ranges: Vec<(usize, usize)>,

    /// otl[circuit_index][outgoing_port_index][i] = destination_port_index
    output_target_list: Vec<Vec<Vec<ConnectionBehavior>>>,

    /// the value to multiply all samples by
    sample_multiplier: f32,
}

#[derive(Debug, Clone, Copy)]
struct ConnectionBehavior {
    /// First bit specifies behavior, rest of bits specify index
    /// First bit is 1 -> Save
    /// First bit is 0 -> Send
    data: usize
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Behavior {
    Send,
    Save
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

impl PlaybackBackendData {
    pub fn new(
        ids: &[CircuitId],
        builders: &HashMap<CircuitId, Box<dyn CircuitBuilder>>,
        connections: &ConnectionManager,
        speakers: &HashSet<CircuitId>,
        sample_multiplier: f32,
    ) -> Self {
        // determine circuit order and depths
        let (circuits, depths) = Self::compute_order(
            builders,
            connections,
            ids.iter()
                .filter_map(|id| {
                    if speakers.contains(id) {
                        Some(*id)
                    } else {
                        None
                    }
                })
                .collect()
        );

        // determine send/save behavior
        let mut circuit_id_to_index_map: HashMap<CircuitId, usize> = HashMap::new();
        let mut port_id_to_index_map: HashMap<CircuitPortId, usize> = HashMap::new();
        let mut input_ranges: Vec<(usize, usize)> = vec![];

        let mut port_index = 0;
        for (circuit_index, circuit) in circuits.iter().enumerate() {
            let start_index = port_index;
            circuit_id_to_index_map.insert(*circuit, circuit_index);
            let spec = builders[circuit].specification();
            for input_port in spec.circuit_input_port_id_iter(*circuit) {
                port_id_to_index_map.insert(input_port, port_index);
                port_index += 1;
            }
            input_ranges.push((start_index, port_index));
        }
        let speaker_index = circuits.len();

        // otl[circuit_index][outgoing_port_index][i] = destination_port_index
        let mut output_target_list = vec![];

        // Iterate over all circuits
        for (circuit_index, circuit) in circuits.iter().enumerate() {
            // opt[outgoing_port_index][i] = destination_port_index
            let mut output_ports = vec![];
            let circuit_depth = depths[circuit_index];
            let spec = builders[circuit].specification();

            // Iterate over all output ports
            for out_port in spec.circuit_output_port_id_iter(*circuit) {
                // ot[i] = destination_port_index
                let mut output_targets = vec![];
                if let Some(destinations) = connections.port_query_ports(out_port) {

                    // Iterate over all output targets
                    // Determine processing behavior for each target
                    for dst in destinations {
                        let dst_circuit = dst.circuit_id;
                        if speakers.contains(&dst_circuit) {
                            output_targets.push(ConnectionBehavior::new(
                                Behavior::Send,
                                speaker_index
                            ));
                        } else {
                            let dst_circuit_index = circuit_id_to_index_map[&dst_circuit];
                            let behavior = if depths[dst_circuit_index] < circuit_depth {
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

        let input_buffer = vec![0.0; speaker_index + 1];

        let mut built_circuits = Vec::with_capacity(circuits.len());
        for circuit_id in circuits {
        	built_circuits.push(builders[&circuit_id].build());
        }

        assert!(
            built_circuits.len() == speaker_index,
            "built_circuits should have a length equal to the speaker index. built_circuits: {}; speaker_index: {}",
            built_circuits.len(),
            speaker_index
        );

        assert!(
            built_circuits.len() == output_target_list.len(),
            "built_circuits should have as many elements as output_target_list. built_circuits: {}; otl: {}",
            built_circuits.len(),
            output_target_list.len()
        );

        assert!(
            built_circuits.len() == input_ranges.len(),
            "built_circuits should have as many elements input_ranges. built_circuits: {}; input_ranges: {}",
            built_circuits.len(),
            input_ranges.len()
        );

        assert!(
            built_circuits.len() + 1 == input_buffer.len(),
            "built_circuits should have a length one less than that of the input buffer. built_circuits: {}; input_buffer: {}",
            built_circuits.len(),
            input_buffer.len()
        );

        Self {
            circuits: built_circuits,
            input_buffer,
            input_ranges,
            output_target_list,
            sample_multiplier
        }
    }

    /// Determine the order and depth of the circuits
    fn compute_order(
        builders: &HashMap<CircuitId, Box<dyn CircuitBuilder>>,
        connections: &ConnectionManager,
        speaker_vec: Vec<CircuitId>
    ) -> (Vec<CircuitId>, Vec<usize>) {
        // the order at which to update circuits, reversed order
        let mut circuits_rev: Vec<CircuitId> = vec![];

        // the distance from each circuit to a speaker, reversed order
        let mut depths_rev: Vec<usize> = vec![];

        // a list of all circuits already visited or queued
        let mut visited: HashSet<CircuitId> = HashSet::new();

        // the current list of circuits to search
        let mut current_queue = speaker_vec;

        // the next list of circuits to search
        let mut next_queue = vec![];

        // the current depth of circuit
        let mut depth = 0;

        while !current_queue.is_empty() {
            let current = current_queue.pop().unwrap();
            'inner: for input_port in builders[&current].specification().circuit_input_port_id_iter(current) {
                let sources = connections.port_query_ports(input_port);
                if sources == None {
                    continue 'inner;
                }
                for source in sources.unwrap() {
                    let source_circuit = source.circuit_id;
                    if !visited.contains(&source_circuit) {
                        visited.insert(source_circuit);
                        circuits_rev.push(source_circuit);
                        depths_rev.push(depth);
                        next_queue.push(source_circuit);
                    }
                }
            }
            if current_queue.is_empty() {
                current_queue = next_queue;
                next_queue = vec![];
            }
            depth += 1;
        }

        circuits_rev.reverse();
        depths_rev.reverse();

        assert!(circuits_rev.len() == depths_rev.len(), "Circuits and depths should be equal");

        (circuits_rev, depths_rev)
    }

    /// Updates all circuits once and in order for one sample
    /// Returns the value of the sample as an f32
    pub fn update(&mut self) -> f32 {
        // the buffer where save-behavior items are stored
        let mut save_buffer = vec![0.0; self.input_buffer.len()];

        // handle internal updates
        for i in 0..self.circuits.len() {
            // the current circuit to update
            let circuit = &mut self.circuits[i];

            // the range of inputs associated with the circuit
            let range = self.input_ranges[i];

            // the slice the circuit should take input from
            let inputs = &self.input_buffer[range.0..range.1];

            // the buffer the circuit should write to
            let mut output_buffer = vec![0.0; self.output_target_list.len()];

            circuit.operate(&inputs, &mut output_buffer);

            // iterate through each output port to send or save the result
            for j in 0..output_buffer.len() {
                // the value stored at the current output slot
                let output_value = output_buffer[j];

                // the list of targets that the output should be sent to
                let targets = &self.output_target_list[i][j];

                // iterate through each output target to send or save the result
                for target in targets {
                    match target.behavior() {
                        Behavior::Send => {
                            self.input_buffer[target.index()] += output_value;
                        }
                        Behavior::Save => {
                            save_buffer[target.index()] += output_value;
                        }
                    }
                }
            }
        }

        // handle speaker output
        let sample = *self.input_buffer.last().unwrap();
        self.input_buffer = save_buffer;
        sample
    }

    /// Converts the backend to a callback used for an audio stream
    pub fn stream_data_callback<T: Sample + FromSample<f32>>(
        mut self
    ) -> impl FnMut(&mut [T], &OutputCallbackInfo) {
        move |data, _callback_info| {
            for sample in data.iter_mut() {
                let raw_sample = self.update() * self.sample_multiplier;
                *sample = Sample::to_sample::<T>(raw_sample);
            }
        }
    }

    /// Creates an output stream, consuming self
    pub fn into_output_stream<D: DeviceTrait, E:FnMut(StreamError) + Send + 'static>(
        self,
        device: &D,
        config: SupportedStreamConfig,
        error_callback: E,
        timeout: Option<Duration>,
    ) -> Result<D::Stream, BuildStreamError> {
        let format = config.sample_format();
        let cfg: StreamConfig = config.into();
        match format {
            SampleFormat::I16 => {
                device.build_output_stream(
                    &cfg,
                    self.stream_data_callback::<i16>(),
                    error_callback,
                    timeout
                )
            },
            SampleFormat::I32 => {
                device.build_output_stream(
                    &cfg,
                    self.stream_data_callback::<i32>(),
                    error_callback,
                    timeout
                )
            },
            SampleFormat::I64 => {
                device.build_output_stream(
                    &cfg,
                    self.stream_data_callback::<i64>(),
                    error_callback,
                    timeout
                )
            },
            SampleFormat::U16 => {
                device.build_output_stream(
                    &cfg,
                    self.stream_data_callback::<u16>(),
                    error_callback,
                    timeout
                )
            },
            SampleFormat::U32 => {
                device.build_output_stream(
                    &cfg,
                    self.stream_data_callback::<u32>(),
                    error_callback,
                    timeout
                )
            },
            SampleFormat::U64 => {
                device.build_output_stream(
                    &cfg,
                    self.stream_data_callback::<u64>(),
                    error_callback,
                    timeout
                )
            },
            SampleFormat::F32 => {
                device.build_output_stream(
                    &cfg,
                    self.stream_data_callback::<f32>(),
                    error_callback,
                    timeout
                )
            },
            SampleFormat::F64 => {
                device.build_output_stream(
                    &cfg,
                    self.stream_data_callback::<f64>(),
                    error_callback,
                    timeout
                )
            },
            _ => panic!("Unsupported stream format.")
        }
    }

}
