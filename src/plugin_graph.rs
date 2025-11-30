use std::collections::{HashMap, VecDeque};

use crate::{live_plugin_id::{LivePluginId, LivePluginKind}, playback::{InputSpecification, LiveDrum, LiveEffect, LiveEffectContainer, LivePlugin, LiveSynth}};

pub struct EffectGraph {
    /// Contains all nodes without children
    childless_nodes: Vec<*mut Node>,

    /// A map from input components (drums or synths) to the nodes that take direct input from them
    input_map: HashMap<LivePluginId, Vec<*mut Node>>,

    /// Special node containing the main audio output
    output_node: *mut Node,

    /// A map from the plugin id to its corresponding node
    id_node_map: HashMap<LivePluginId, *mut Node>,

    /// tracks the total number of managed effects
    effect_count: u32,
}

struct Node {
    /// the id of the corresponding effect
    id: LivePluginId,

    /// a list of direct inputs restricted to synth and drum components
    inputs: Vec<LivePluginId>,

    /// a list of parent (preceeding) nodes
    parents: Vec<*mut Node>,

    /// a list of child (succeeding) nodes
    children: Vec<*mut Node>,
}

impl Node {
    fn new(effect: LivePluginId) -> Self {
        Self {
            id: effect,
            inputs: Vec::new(),
            parents: Vec::new(),
            children: Vec::new(),
        }
    }

    /// adds the input with the given id to the list of inputs
    /// returns true if adding was successful
    fn add_input(&mut self, input: LivePluginId) -> bool {
        match self.inputs.binary_search(&input) {
            Ok(_) => {
                false
            },
            Err(index) => {
                self.inputs.insert(index, input);
                true
            }
        }
    }

    /// adds the parent with the given pointer to the list of parents
    /// returns true if adding was successful
    fn add_parent(&mut self, node: *mut Node) -> bool {
        match self.parents.binary_search(&node) {
            Ok(_) => {
                false
            },
            Err(index) => {
                self.parents.insert(index, node);
                true
            }
        }
    }

    /// adds the child with the given pointer to the list of children
    /// returns true if adding was successful
    fn add_child(&mut self, node: *mut Node) -> bool {
        match self.children.binary_search(&node) {
            Ok(_) => {
                false
            },
            Err(index) => {
                self.children.insert(index, node);
                true
            }
        }
    }

    /// removes the input with the given id from the list of parents
    /// returns true if the removal was successful
    fn remove_input(&mut self, input: LivePluginId) -> bool {
        match self.inputs.binary_search(&input) {
            Ok(index) => {
                self.inputs.remove(index);
                true
            },
            Err(_) => {
                false
            }
        }
    }

    /// removes the parent with the given pointer from the list of parents
    /// returns true if the removal was successful
    fn remove_parent(&mut self, node: *mut Node) -> bool {
        match self.parents.binary_search(&node) {
            Ok(index) => {
                self.parents.remove(index);
                true
            },
            Err(_) => {
                false
            }
        }
    }

    /// removes the child with the given pointer from the list of children
    /// returns true if the removal was successful
    fn remove_child(&mut self, node: *mut Node) -> bool {
        match self.children.binary_search(&node) {
            Ok(index) => {
                self.children.remove(index);
                true
            },
            Err(_) => {
                false
            }
        }
    }

    /// returns true if the node has no parents
    fn is_orphaned(&self) -> bool {
        self.parents.is_empty() && self.inputs.is_empty()
    }

    /// returns true if the node has no children
    fn is_childless(&self) -> bool {
        self.children.is_empty()
    }
}

impl EffectGraph {
    /// creates a new effect graph
    pub fn new() -> Self {
        let output_node = Box::into_raw(Box::new(Node::new(LivePluginId::NIL)));
        Self {
            output_node,
            childless_nodes: Vec::new(),
            input_map: HashMap::new(),
            id_node_map: HashMap::new(),
            effect_count: 0,
        }
    }

    /// removes a node from the list of childless nodes
    /// returns true if the removal is successful
    fn remove_childless(&mut self, node: *mut Node) -> bool {
        let index = self.childless_nodes.binary_search(&node);
        match index {
            // node is stored as childless
            Ok(i) => {
                self.childless_nodes.remove(i);
                true
            }

            // node is not childless
            Err(_) => {
                false
            }
        }
    }

    /// inserts a node into the list of childless nodes
    /// returns true if the insert is successful
    fn insert_childless(&mut self, node: *mut Node) -> bool {
        let index = self.childless_nodes.binary_search(&node);
        match index {
            // node is already registered as childless
            Ok(_) => false,

            // node is new
            Err(i) => {
                self.childless_nodes.insert(i, node);
                true
            }
        }
    }

    /// adds a node to the global list of input targets
    fn register_input(&mut self, input: LivePluginId, node: *mut Node) -> bool {
        if !self.input_map.contains_key(&input) {
            self.input_map.insert(input.clone(), Vec::new());
        }
        let list = self.input_map.get_mut(&input).unwrap();
        let index = list.binary_search(&node);
        match index {
            // node is registered already
            Ok(_) => false,

            // node is new
            Err(index) => {
                list.insert(index, node);
                true
            }
        }
    }

    /// removes a node from the global list of input targets
    fn unregister_input(&mut self, input: LivePluginId, node: *mut Node) -> bool {
        if let Some(list) = self.input_map.get_mut(&input) {
            let index = list.binary_search(&node);
            match index {
                // node is registered
                Ok(index) => {
                    list.remove(index);
                    if list.is_empty() {
                        self.input_map.remove(&input);
                    }
                    true
                }

                // node is new
                Err(_) => false,
            }
        } else {
            false
        }
    }

    /// adds a component to the graph with the given id
    pub fn add_effect(
        &mut self,
        id: LivePluginId
    ) {
        // insert node into node map
        debug_assert!(self.id_node_map.contains_key(&id), "Attempted to add an effect that is already registered");
        let node = Box::into_raw(Box::new(Node::new(id)));
        self.id_node_map.insert(id, node);

        // register as a childless node
        self.insert_childless(node);

        // if code is changed, be sure to check that duplicates are not added before incrementing
        self.effect_count += 1;
    }

    /// removes a component from the graph with the given id
    /// returns the container the effect was stored in
    pub fn remove_effect(
        &mut self,
        id: LivePluginId
    ) {
        // remove node from node map
        debug_assert!(!self.id_node_map.contains_key(&id), "Attempted to remove an effect that does not exist");
        let node = self.id_node_map.remove(&id).unwrap();

        unsafe {
            // update parents's lists of children
            for parent_raw in &(*node).parents {
                let parent = parent_raw.as_mut().unwrap();
                parent.remove_child(node);

                // if parent becomes childless, list it as one
                if parent.is_childless() {
                    self.insert_childless(*parent_raw);
                }
            }
        }

        unsafe {
            // remove input references
            for input in &(*node).inputs {
                self.unregister_input(*input, node);
            }
        }

        unsafe {
            if (*node).children.is_empty() {
                // remove from this node from the global list of childless nodes
                self.childless_nodes.retain(|n| *n != node);
            } else {
                // update children's lists of parents
                for child in &(*node).children {
                    child.as_mut().unwrap().remove_parent(node);
                }
            }
        }

        unsafe { node.drop_in_place() }

        // if code is changed, be sure to check that non-existent effects are not removed before
        // decrementing
        self.effect_count -= 1;
    }

    /// creates a new connection between effects
    pub fn connect_effects(&mut self, src: LivePluginId, dst: LivePluginId) {
        let src_node_raw = self.id_node_map.get(&src);
        let dst_node_raw = self.id_node_map.get(&dst);
        debug_assert!(src_node_raw.is_some(), "Source effect is not stored in this graph");
        debug_assert!(dst_node_raw.is_some(), "Destination effect is not stored in this graph");
        let src_node = *src_node_raw.unwrap();
        let dst_node = *dst_node_raw.unwrap();

        // update list of childless nodes
        if unsafe { (*src_node).is_childless() } {
            self.remove_childless(src_node);
        }

        // update child and parent lists
        unsafe { (*src_node).add_child(dst_node); }
        unsafe { (*dst_node).add_parent(src_node); }
    }

    /// removes a connection between effects
    pub fn disconnect_effects(&mut self, src: LivePluginId, dst: LivePluginId) {
        let src_node_raw = self.id_node_map.get(&src);
        let dst_node_raw = self.id_node_map.get(&dst);
        debug_assert!(src_node_raw.is_some(), "Source effect is not stored in this graph");
        debug_assert!(dst_node_raw.is_some(), "Destination effect is not stored in this graph");
        let src_node = *src_node_raw.unwrap();
        let dst_node = *dst_node_raw.unwrap();

        // update child and parent lists
        unsafe { (*src_node).remove_child(dst_node); }
        unsafe { (*dst_node).remove_parent(src_node); }

        // check if this disconnection made the source node childless
        if unsafe { (*src_node).is_childless() } {
            self.insert_childless(src_node);
        }
    }

    /// connects an effect to the main output of the effect graph
    pub fn connect_output(&mut self, src: LivePluginId) {
        let src_node_raw = self.id_node_map.get(&src);
        debug_assert!(src_node_raw.is_some(), "Source effect is not stored in this graph");
        let src_node = *src_node_raw.unwrap();

        if unsafe { (*src_node).is_childless() } {
            self.remove_childless(src_node);
        }

        unsafe { (*src_node).add_child(self.output_node); }
    }

    /// disconnects an effect from the main output of the effect graph
    pub fn disconnect_output(&mut self, src: LivePluginId) {
        let src_node_raw = self.id_node_map.get(&src);
        debug_assert!(src_node_raw.is_some(), "Source effect is not stored in this graph");
        let src_node = *src_node_raw.unwrap();

        // update child and parent lists
        unsafe { (*src_node).remove_child(self.output_node); }

        // check if this disconnection made the source node childless
        if unsafe { (*src_node).is_childless() } {
            self.insert_childless(src_node);
        }
    }

    /// connects an input to an effect in the graph
    pub fn connect_input(&mut self, src: LivePluginId, dst: LivePluginId) {
        debug_assert!(
            src.kind() == LivePluginKind::Synth || src.kind() == LivePluginKind::Drum,
            "Attempted to connect input that is not a drum or synth."
        );
        let dst_node_raw = self.id_node_map.get(&dst);
        debug_assert!(dst_node_raw.is_some(), "Destination effect is not stored in this graph");
        let dst_node = *dst_node_raw.unwrap();

        unsafe { (*dst_node).add_input(src) };
        self.register_input(src, dst_node);
    }

    /// disconnects an input from an effect in the graph
    pub fn disconnect_input(&mut self, src: LivePluginId, dst: LivePluginId) {
        debug_assert!(
            src.kind() == LivePluginKind::Synth || src.kind() == LivePluginKind::Drum,
            "Attempted to disconnect input that is not a drum or synth."
        );
        let dst_node_raw = self.id_node_map.get(&dst);
        debug_assert!(dst_node_raw.is_some(), "Destination effect is not stored in this graph");
        let dst_node = *dst_node_raw.unwrap();

        unsafe { (*dst_node).remove_input(src) };
        self.unregister_input(src, dst_node);
    }

    /// connects an input directly to the output of the graph (without effects applied)
    pub fn connect_direct_input(&mut self, src: LivePluginId) {
        debug_assert!(
            src.kind() == LivePluginKind::Synth || src.kind() == LivePluginKind::Drum,
            "Attempted to connect input that is not a drum or synth."
        );

        unsafe { (*self.output_node).add_input(src) };
        self.register_input(src, self.output_node);
    }

    /// disconnects a direct input from the output of the graph
    pub fn disconnect_direct_input(&mut self, src: LivePluginId) {
        debug_assert!(
            src.kind() == LivePluginKind::Synth || src.kind() == LivePluginKind::Drum,
            "Attempted to disconnect input that is not a drum or synth."
        );

        unsafe { (*self.output_node).remove_input(src) };
        self.unregister_input(src, self.output_node);
    }

    /// checks if the graph manages the component with the given id
    pub fn is_managed(&self, id: LivePluginId) -> bool {
        self.id_node_map.contains_key(&id) || self.input_map.contains_key(&id)
    }

    /// overwrites the given playback order with the order for this graph
	pub fn overwrite_order(
        &self,
        order: &mut EffectGraphOrder,
        effect_map: &HashMap<LivePluginId, *mut LiveEffectContainer>
    ) {
        // a map from each plugin to (depth, finish_time)
        // depth is recorded as path length to the output
        // minimum depth for a non-output node is 1
        let mut depth_map = HashMap::new();
        let mut current_queue: Vec<*mut Node> = Vec::new();
        let mut next_queue: Vec<*mut Node> = Vec::new();
        let mut id_order: VecDeque<LivePluginId> = VecDeque::new();

        depth_map.insert(LivePluginId::NIL, (0, 0));

        {
            let mut finish_time = 1;
            for node in unsafe { &(*self.output_node).parents } {
                let id = unsafe { (**node).id };
                depth_map.insert(id, (1, finish_time));
                current_queue.push(*node);
                id_order.push_front(id);
                finish_time += 1;
            }

            let mut depth = 2;

            // add nodes with path to output
            while !current_queue.is_empty() {
                let node = current_queue.pop().unwrap();
                for parent in unsafe { &(*node).parents } {
                    let id = unsafe { (**parent).id };
                    if !depth_map.contains_key(&id) {
                        depth_map.insert(id, (depth, finish_time));
                        next_queue.push(*parent);
                        id_order.push_front(id);
                        finish_time += 1;
                    }
                }

                if current_queue.is_empty() {
                    (current_queue, next_queue) = (next_queue, current_queue);
                    depth += 1;
                }
            }

            // add nodes without path to output
            for node in &self.childless_nodes {
                let id = unsafe { (**node).id };
                depth_map.insert(id, (1, finish_time));
                current_queue.push(*node);
                id_order.push_front(id);
                finish_time += 1;
            }

            depth = 2;

            while !current_queue.is_empty() {
                let node = current_queue.pop().unwrap();
                for parent in unsafe { &(*node).parents } {
                    let id = unsafe { (**parent).id };
                    if !depth_map.contains_key(&id) {
                        depth_map.insert(id, (depth, finish_time));
                        next_queue.push(*parent);
                        id_order.push_front(id);
                        finish_time += 1;
                    }
                }

                if current_queue.is_empty() {
                    (current_queue, next_queue) = (next_queue, current_queue);
                    depth += 1;
                }
            }
        }

        // data associated with the playback order being returned
        let mut targets = Vec::new();
        let mut effects = Vec::new();

        // scratch buffer for storing effects with save behavior
        // at the start/end of each iteration in the following loop,
        // we must ensure that the scratch buffer is empty
        let mut scratch = Vec::new();

        for effect_id in &id_order {
            let (depth, finish_time) = depth_map[effect_id];
            effects.push(effect_map[effect_id]);

            let mut send_save_buffer_data = Vec::new();
            for child in unsafe { &(*self.id_node_map[effect_id]).children } {
                let child_id = unsafe { (**child).id };
                let (child_depth, child_finish_time) = depth_map[&child_id];
                let child_effect = if child_id.is_nil() {
                    order.output
                } else {
                    effect_map[&child_id]
                };
                if depth == child_depth && finish_time > child_finish_time {
                    // if we are at an equal distance from an output, but come before it, we need
                    // to use save behavior
                    scratch.push(child_effect);
                } else {
                    send_save_buffer_data.push(child_effect);
                }
            }
            let start_save = send_save_buffer_data.len();
            while !scratch.is_empty() {
                send_save_buffer_data.push(scratch.pop().unwrap());
            }
            targets.push(EffectSendSaveBuffer { start_save, data: send_save_buffer_data });
        }

        debug_assert!(effects.len() == targets.len(), "We must have an equal number of effects as targets");

        order.targets = targets;
        order.effects = effects;
    }

}

pub struct PlaybackOrder {
    /// drums and their sends
    pub(super) drums: Vec<*mut dyn LiveDrum>,
    pub(super) drum_sends: Vec<Vec<*mut LiveEffectContainer>>,

    /// synths and their sends
    pub(super) synths: Vec<*mut dyn LiveSynth>,
    pub(super) synth_sends: Vec<Vec<*mut LiveEffectContainer>>,

    /// the effect groups
    pub(super) effect_groups: Vec<Box<EffectGraphOrder>>,

    /// the main output
    pub(super) main_output: *mut LiveEffectContainer
}

impl PlaybackOrder {
    /// updates all components and gets the output
    pub fn update(&self, sample_rate: u32) -> f32 {
        // update drums
        for (drum, sends) in self.drums.iter().zip(self.drum_sends.iter()) {
            let sample = unsafe { (**drum).update(sample_rate) };
            for send in sends {
                unsafe { (**send).send(sample); }
            }
        }

        // update synths
        for (synth, sends) in self.synths.iter().zip(self.synth_sends.iter()) {
            let sample = unsafe { (**synth).update(sample_rate) };
            for send in sends {
                unsafe { (**send).send(sample); }
            }
        }

        // update effects
        for group in &self.effect_groups {
            unsafe { (*self.main_output).send((**group).update(sample_rate)) };
        }

        // get main output
        unsafe { (*self.main_output).update(sample_rate) }
    }

    /// removes the effect group with the given id
    /// returns true if the removal was successful
    pub fn remove_group(&mut self, id: LivePluginId) -> bool {
        let index = self.effect_groups.binary_search_by(|g| g.id.cmp(&id));
        match index {
            Ok(i) => {
                self.effect_groups.remove(i);
                true
            },
            Err(_) => false
        }
    }

    /// creates an effect group with the given id
    /// returns true if the removal was successful
    pub fn add_group(&mut self, id: LivePluginId) -> bool {
        let index = self.effect_groups.binary_search_by(|g| g.id.cmp(&id));
        match index {
            Err(i) => {
                self.effect_groups.insert(i, Box::new(EffectGraphOrder::new(id)));
                true
            },
            Ok(_) => false
        }
    }

    /// modifies the effect group using data from the given effect graph
    /// returns true on a successful modificiation
    pub fn modify_group(
        &mut self,
        id: LivePluginId,
        graph: &EffectGraph,
        effect_map: &HashMap<LivePluginId, *mut LiveEffectContainer>,
    ) -> bool {
        if let Some(mut order) = self.get_group_mut(id) {
            graph.overwrite_order(&mut order, effect_map);
            true
        } else {
            false
        }
    }

    /// gets the group with the given id mutably
    fn get_group_mut(&mut self, id: LivePluginId) -> Option<&mut EffectGraphOrder> {
        let index = self.effect_groups.binary_search_by(|g| g.id.cmp(&id));
        match index {
            Ok(i) => {
                Some(&mut self.effect_groups[i])
            },
            Err(_) => None
        }
    }

    /// gets the group with the given id immutably
    pub fn get_group(&mut self, id: LivePluginId) -> Option<&EffectGraphOrder> {
        let index = self.effect_groups.binary_search_by(|g| g.id.cmp(&id));
        match index {
            Ok(i) => {
                Some(&self.effect_groups[i])
            },
            Err(_) => None
        }
    }
}

#[derive(Debug)]
pub struct EffectGraphOrder {
    effects: Vec<*mut LiveEffectContainer>,
    targets: Vec<EffectSendSaveBuffer>,
    output: *mut LiveEffectContainer,
    id: LivePluginId,
}

#[derive(Debug)]
struct EffectSendSaveBuffer {
    /// the index at which to start using save behavior
    start_save: usize,
    data: Vec<*mut LiveEffectContainer>
}

impl EffectGraphOrder {
    const MIN_VOLUME: f32 = 0.0;
    const MAX_VOLUME: f32 = 2.0;

    pub fn output_effect(&self) -> *mut LiveEffectContainer {
        self.output
    }

    pub fn id(&self) -> LivePluginId {
        self.id
    }

    pub fn new(id: LivePluginId) -> Self {
        let output_effect = Box::new(EffectGroupOutput::new());
        let output = unsafe { Box::into_raw(Box::new(LiveEffectContainer::new(output_effect))) };

        Self {
            effects: Vec::new(),
            targets: Vec::new(),
            output,
            id
        }
    }

    // updates effects and sends outputs to targets
    // returns the sample following updates
    // safety: you must ensure that all of the contained effects are valid
    pub unsafe fn update(&self, sample_rate: u32) -> f32 {
        for (effect, target) in self.effects.iter().zip(self.targets.iter()) {
            let sample = unsafe { (**effect).update(sample_rate) };
            for i in 0..target.start_save {
                unsafe { target.data[i].as_mut().unwrap().send(sample); }
            }
            for i in target.start_save..self.targets.len() {
                unsafe { target.data[i].as_mut().unwrap().save(sample); }
            }
        }

        unsafe { (*self.output).update(sample_rate) }
    }
}

#[derive(Debug)]
pub struct EffectGroupOutput {
    /// fractional volume of output
    volume: f32,

    /// whether or not output is muted
    muted: bool
}

impl LivePlugin for EffectGroupOutput {
    fn reset(&mut self) {
        self.volume = 1.0;
        self.muted = false;
    }

    fn get_inputs(&self) -> Vec<InputSpecification> {
        vec![
            InputSpecification {
                id: 0,
                name: "Volume".to_string(),
                short_name: "Vol".to_string(),
                range: (0.0, 1.5),
                input_values: 0,
                default: 1.0
            },
            InputSpecification {
                id: 1,
                name: "Muted".to_string(),
                short_name: "Mute".to_string(),
                range: (0.0, 1.0),
                input_values: 2,
                default: 0.0
            },
        ]
    }

    fn set_input(&mut self, id: crate::playback::InputId, value: f64) {
        match id {
            0 => { self.volume = value as f32; }

            1 => { self.muted = value >= 0.5; }

            _ => unreachable!("It should be guaranteed that only ids of 0 and 1 are arguments.")
        }
    }
}

impl LiveEffect for EffectGroupOutput {
    fn update(&mut self, sample: f32, _sample_rate: u32) -> f32 {
        if self.muted {
            0.0
        } else {
            sample * self.volume
        }
    }

}

impl EffectGroupOutput {
    pub fn new() -> Self {
        Self {
            volume: 1.0,
            muted: false
        }
    }
}

impl Default for EffectGroupOutput {
    fn default() -> Self {
        Self::new()
    }
}
