use std::{collections::{HashMap, HashSet}, u16};
use egui::{Color32, Painter, Pos2, Stroke, epaint::CubicBezierShape};

use crate::{circuit_id::{CircuitPortId, ConnectionId}, circuit::PortUi};

///The amount of possible colors for a connection
pub const CONNECTION_COLOR_COUNT: usize = 5;
type ColorIndex = u8;
type ColorCount = u16;
type ColorCounter = [ColorCount; CONNECTION_COLOR_COUNT];

trait ColorTracker {
    ///Adds a color with the given index to the tracker
    fn add_color(&mut self, index: ColorIndex);

    ///Removes a color with the given index from the tracker
    fn remove_color(&mut self, index: ColorIndex);

    ///Gets the least common colors from a pair of trackers
    fn min_colors(&self, other: &Self) -> Vec<ColorIndex>;

    ///Gets the total number of connections
    fn total_connections(&self) -> usize;
}

impl ColorTracker for ColorCounter {
    fn add_color(&mut self, index: ColorIndex) {
        assert!((index as usize) < CONNECTION_COLOR_COUNT);
        self[index as usize] += 1;
    }

    fn remove_color(&mut self, index: ColorIndex) {
        assert!((index as usize) < CONNECTION_COLOR_COUNT);
        self[index as usize] = self[index as usize].saturating_sub(1);
    }

    fn min_colors(&self, other: &Self) -> Vec<ColorIndex> {
        let mut min = self[0] + other[0];
        for (count1, count2) in self.iter().zip(other.iter()) {
            let count = count1 + count2;
            if count < min {
                min = count;
            }
        };
        let mut output = vec![];
        for (index, (count1, count2)) in self.iter().zip(other.iter()).enumerate() {
            let count = count1 + count2;
            if count <= min {
                output.push(index as ColorIndex);
            }
        };
        output
    }

    fn total_connections(&self) -> usize {
        let mut sum: usize = 0;
        for count in self {
            sum += *count as usize;
        }
        sum
    }

}

///The array of possible colors for a connection
pub const CONNECTION_COLORS: [Color32; CONNECTION_COLOR_COUNT] = [
    Color32::RED,
    Color32::YELLOW,
    Color32::GREEN,
    Color32::CYAN,
    Color32::MAGENTA,
];

#[derive(Debug, Default)]
pub struct ConnectionManager {
    ///The list of all connections and their assigned colors
    connections: Vec<(ConnectionId, ColorIndex)>,

    ///The set of all connections
    connection_set: HashSet<ConnectionId>,

    ///A map matching a port to the colors of its associated connections
    counter_map: HashMap<CircuitPortId, ColorCounter>,

    ///A map matching a port to the other ports it is connected to
    connection_map: HashMap<CircuitPortId, Vec<CircuitPortId>>,

    ///A number used to determine the next connection color for variety
    next_color: usize
}

impl ConnectionManager {
    ///Adds the given connection to the list of connections.
    ///Returns true if the connection was successfully added.
    pub fn add_connection(&mut self, connection: ConnectionId) -> bool {
        if !self.connection_set.contains(&connection) {
            //ensure color counters are initialized
            if !self.counter_map.contains_key(&connection.src()) {
                self.counter_map.insert(connection.src(), ColorCounter::default());
            }
            if !self.counter_map.contains_key(&connection.dst()) {
                self.counter_map.insert(connection.dst(), ColorCounter::default());
            }

            //ensure connection maps are initialized
            if !self.connection_map.contains_key(&connection.src()) {
                self.connection_map.insert(connection.src(), vec![]);
            }
            if !self.connection_map.contains_key(&connection.dst()) {
                self.connection_map.insert(connection.dst(), vec![]);
            }

            //pick the color for this connection
            let color = {
                let colors = self.counter_map.get(&connection.src()).unwrap().min_colors(
                    self.counter_map.get(&connection.dst()).unwrap()
                );
                let index = self.next_color % colors.len();
                self.next_color += 1;
                colors[index]
            };

            //store connection
            //unwrap is safe because we ensured connection maps are initialized
            self.connections.push((connection, color));
            self.connection_set.insert(connection);
            self.connection_map
                .get_mut(&connection.src())
                .unwrap()
                .push(connection.dst());
            self.connection_map
                .get_mut(&connection.dst())
                .unwrap()
                .push(connection.src());

            //update color counters
            //unwrap is safe because we ensured color counters are initialized
            self.counter_map
                .get_mut(&connection.src())
                .unwrap()
                .add_color(color);
            self.counter_map
                .get_mut(&connection.dst())
                .unwrap()
                .add_color(color);

            true
        } else {
            false
        }
    }

    ///Removes the given connection from the list of connections.
    ///Returns true if the connection was successfully removed
    pub fn remove_connection(&mut self, connection: ConnectionId) -> bool {
        if self.connection_set.contains(&connection) {

            //wipe data
            let mut color = ColorIndex::default(); //save for removal later
            self.connections.retain(|(entry, col)| {
                if *entry == connection {
                    color = *col;
                    false
                } else {
                    true
                }
            });
            self.connection_set.remove(&connection);
            self.connection_map
                .get_mut(&connection.src())
                .unwrap()
                .retain(|port| *port != connection.dst());
            self.connection_map
                .get_mut(&connection.dst())
                .unwrap()
                .retain(|port| *port != connection.src());

            //remove color
            self.counter_map
                .get_mut(&connection.src())
                .unwrap()
                .remove_color(color);
            self.counter_map
                .get_mut(&connection.dst())
                .unwrap()
                .remove_color(color);

            true
        } else {
            false
        }
    }

    const CONNECT_Y_FACTOR: f32 = 1000.0;
    const CONNECT_POSITIVE_X_FACTOR: f32 = 1.0 / 1.5;
    const CONNECT_NEGATIVE_X_FACTOR: f32 = 0.5;
    const CONNECT_MIN_X: f32 = 100.0;
    const CONNECT_MAX_X: f32 = 200.0;
    const CONNECT_THICKNESS: f32 = 1.0;
    ///gets the points for the cubic bezier connecting the start and end points
    fn get_connection_points(start: Pos2, end: Pos2) -> [Pos2; 4] {
        let mut diff_x = (end.x - start.x).abs();
        let mut diff_y = 0.0;
        if start.x > end.x {
            diff_y = (end.y - start.y) / Self::CONNECT_Y_FACTOR * diff_x.min(Self::CONNECT_Y_FACTOR);
            diff_x = Self::CONNECT_MIN_X + (diff_x * Self::CONNECT_NEGATIVE_X_FACTOR).min(Self::CONNECT_MAX_X);
        } else {
            diff_x *= Self::CONNECT_POSITIVE_X_FACTOR;
        }
        diff_x = diff_x.max(Self::CONNECT_MIN_X);
        [
            start,
            egui::pos2(start.x + diff_x, start.y + diff_y),
            egui::pos2(end.x - diff_x, end.y - diff_y),
            end
        ]
    }


    ///draws the connection between two points
    pub fn draw_connection(painter: &Painter, color: Color32, start: Pos2, end: Pos2) {
        let connection = CubicBezierShape::from_points_stroke(
            Self::get_connection_points(start, end),
            false,
            Color32::TRANSPARENT,
            Stroke::new(Self::CONNECT_THICKNESS, color)
        );
        painter.add(connection);
        painter.circle_filled(start, PortUi::FILLED_RADIUS, PortUi::FILLED_COLOR);
        painter.circle_filled(end, PortUi::FILLED_RADIUS, PortUi::FILLED_COLOR);
    }

    ///Draws all connections to the screen, suing the given map of positions
    pub fn draw_connections(&self, painter: &Painter, positions: &HashMap<CircuitPortId, Pos2>) {
        for (connection, color_idx) in &self.connections {
            Self::draw_connection(
                painter, 
                CONNECTION_COLORS[*color_idx as usize],
                positions[&connection.src()],
                positions[&connection.dst()],
            );
        }
    }

    ///Returns a slice of all connected ports
    pub fn query_connected(&self, port: CircuitPortId) -> Option<&[CircuitPortId]> {
        return match self.connection_map.get(&port) {
            Some(vec) => Some(vec.as_slice()),
            None => None
        }
    }
}


