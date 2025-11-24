use std::collections::{HashMap, HashSet, VecDeque};

use crate::{circuit_id::{ConnectionId, GlobalPortId}, live_plugin_id::LivePluginId};

/// The representation of plugins during playback
/// This structure is optimized specifically for playback and should only be used on audio
/// processing threads
#[derive(Debug, Default)]
pub struct PlaybackTree {
    /// a collection of all special source nodes
    sources: Vec<*mut TreeNode>,

    /// a list of all vertices in the order that they must be traversed
    vertices: Vec<LivePluginId>,

    /// a map from each id to its (treenode, list of parent nodes)
    node_map: HashMap<LivePluginId, (*mut TreeNode, Vec<*mut TreeNode>)>,
}

#[derive(Debug)]
struct TreeNode {
    id: LivePluginId,
    children: Vec<Vec<GlobalPortId<LivePluginId>>>,
}

impl PlaybackTree {
    pub fn new() -> Self {
        Self::default()
    }

    /// adds a new vertex
    /// returns true if a vertex has been added
    pub fn add_vertex(&mut self, id: LivePluginId) -> bool {
        if self.node_map.contains_key(&id) {
            return false;
        }

        self.vertices.push(id);
        self.node_map.insert(
            id,
            (
                Box::into_raw(Box::new(TreeNode {
                    id,
                    children: Vec::new()
                })),
                Vec::new()
            )
        );
        true
    }

    /// adds a new edge
    /// returns true if a new edge has been added
    pub fn add_edge(&mut self, edge: ConnectionId<LivePluginId>) -> bool {
        if !self.node_map.contains_key(&edge.src().unit_id) || !self.node_map.contains_key(&edge.dst().unit_id) {
            return false;
        }

        let src_node = self.node_map[&edge.src().unit_id].0;
        let dst_node = self.node_map[&edge.dst().unit_id].0;

        let parent_list = &mut self.node_map.get_mut(&edge.dst().unit_id).unwrap().1;
        if parent_list.contains(&dst_node) {
                parent_list.push(src_node);
        }

        let children = unsafe {&mut (*src_node).children};
        while children.len() <= edge.src().port_id.index() {
            children.push(Vec::new());
        }
        children[edge.src().port_id.index()].push(edge.dst());

        true
    }

    /// removes a vertex
    /// returns true if the vertex was removed
    pub fn remove_vertex(&mut self, id: LivePluginId) -> bool {
        if !self.node_map.contains_key(&id) {
            return false;
        }

        let (node, parents) = self.node_map.get_mut(&id).unwrap();

        // remove from sources
        self.sources.retain(|n| n != node);
        
        // remove from parents
        for &mut parent in parents.iter_mut() {
            for port_targets in unsafe {(*parent).children.iter_mut()} {
                port_targets.retain(|p| p.unit_id != id);
            }
        }

        // delete entry
        self.node_map.remove(&id);
        self.vertices.retain(|i| *i != id);

        true
    }

    /// removes an edge
    /// returns true if the edge was removed
    pub fn remove_edge(&mut self, edge: ConnectionId<LivePluginId>) -> bool {
        if !self.node_map.contains_key(&edge.src().unit_id) {
            return false;
        }

        let parent = self.node_map[&edge.src().unit_id].0;
        let targets = unsafe {&mut (*parent).children[edge.src().port_id.index()]};
        for i in 0..targets.len() {
            if targets[i] == edge.dst() {
                targets.remove(i);
                return true;
            }
        }

        false
    }

    /// updates all circuits 
    pub fn update(&self) {
        let mut handled = HashSet::new();

        let mut queue = VecDeque::from_iter(self.sources.iter().cloned());

        for &plugin in &queue {
            handled.insert(unsafe {&*plugin}.id);
        }

        while let Some(plugin) = queue.pop_front() {
            println!("TODO: update {:?}", plugin);
            for port in unsafe {&(*plugin).children} {
                for target in port {
                    let id = target.unit_id;
                    if handled.insert(id) {
                        queue.push_back(self.node_map[&id].0);
                        println!("TODO: handle send behavior");
                    } else {
                        println!("TODO: handle save behavior");
                    }
                }
            }
        }
    }

}
