use crate::{circuit::CircuitBuilderFrontend, circuit_id::ConnectionId};
use std::collections::HashSet;

#[derive(Debug, Default)]
pub struct PatchEditor {
    circuit_builders: Vec<CircuitBuilderFrontend>,
    area_positions: Vec<egui::Pos2>,
    connections: Vec<ConnectionId>,
    connection_set: HashSet<ConnectionId>,
}

impl PatchEditor {
    ///Creates a new PatchEditor. Same as PatchEditor::default()
    pub fn new() -> Self {
        Self::default()
    }

    ///Adds the given connection to the list of connections.
    ///Returns true if the connection was successfully added.
    ///Adds the given connection to the list of connections.
    ///Returns true if the connection was successfully added.
    pub fn add_connection(&mut self, connection: ConnectionId) -> bool {
        if !self.connection_set.contains(&connection) {
            self.connections.push(connection);
            self.connection_set.insert(connection);
            true
        } else {
            false
        }
    }

    ///Removes the given connection from the list of connections.
    ///Returns true if the connection was successfully removed
    pub fn remove_connection(&mut self, connection: ConnectionId) -> bool {
        if self.connection_set.contains(&connection) {
            self.connections.retain(|entry| {
                *entry != connection 
            });
            self.connection_set.remove(&connection);
            true
        } else {
            false
        }
    }
}
