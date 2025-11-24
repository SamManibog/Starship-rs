pub type CircuitIdManager = crate::IdManager<CircuitId>;

pub type CircuitId = u32;

///Designator for an input or output port
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortKind {
    Input,
    Output
}

///The identifier of a port
///has two components: index and kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PortId {
    data: i32,
}

impl PortId {
    pub fn new(index: usize, kind: PortKind) -> Self {
        let sign: i32 = if kind == PortKind::Input { 1 } else { -1 };
        let magnitude: i32 = index as i32 + 1;
        Self {
            data: sign * magnitude
        }
    }

    pub fn kind(&self) -> PortKind {
        if self.data > 0 {
            PortKind::Input
        } else {
            PortKind::Output
        }
    }

    pub fn index(&self) -> usize {
        (self.data.abs() - 1) as usize
    }
}

pub type CircuitPortId = GlobalPortId<CircuitId>;

///The identifier for a port on a specific circuit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalPortId<T: Clone + Copy + PartialEq + Eq + std::hash::Hash + Ord + PartialOrd> {
    pub unit_id: T,
    pub port_id: PortId,
}

impl<T: Clone + Copy + PartialEq + Eq + std::hash::Hash + Ord + PartialOrd> GlobalPortId<T> {
    pub fn new(circuit_id: T, port_id: PortId) -> Self {
        Self {
            unit_id: circuit_id,
            port_id
        }
    }
}

impl<T: Clone + Copy + PartialEq + Eq + std::hash::Hash + Ord + PartialOrd> std::cmp::PartialOrd for GlobalPortId<T> {
    /// ordering is implemented such that unit_id has precedence over port_id
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Clone + Copy + PartialEq + Eq + std::hash::Hash + Ord + PartialOrd> std::cmp::Ord for GlobalPortId<T> {
    /// ordering is implemented such that unit_id has precedence over port_id
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let unit_cmp = self.unit_id.cmp(&other.unit_id);
        if unit_cmp == std::cmp::Ordering::Equal {
            self.port_id.cmp(&other.port_id)
        } else {
            unit_cmp
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionId<T: Clone + Copy + PartialEq + Eq + std::hash::Hash + Ord + PartialOrd> {
    src: GlobalPortId<T>,
    dst: GlobalPortId<T>,
}

impl<T: Clone + Copy + PartialEq + Eq + std::hash::Hash + Ord + PartialOrd> std::cmp::PartialOrd for ConnectionId<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Clone + Copy + PartialEq + Eq + std::hash::Hash + Ord + PartialOrd> std::cmp::Ord for ConnectionId<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let src_cmp = self.src.cmp(&other.src);
        if src_cmp == std::cmp::Ordering::Equal {
            self.dst.cmp(&other.dst)
        } else {
            src_cmp
        }
    }
}

impl<T: Clone + Copy + PartialEq + Eq + std::hash::Hash + Ord + PartialOrd> ConnectionId<T> {
    /// Creates a connection Id where source and destination are specified
    pub fn new(src: GlobalPortId<T>, dst: GlobalPortId<T>) -> Self {
        debug_assert!(src.port_id.kind() == PortKind::Output);
        debug_assert!(dst.port_id.kind() == PortKind::Input);
        Self {
            src,
            dst,
        }
    }

    /// Creates a connection id, automatically determinging the source and destination
    pub fn new_auto(port1: GlobalPortId<T>, port2: GlobalPortId<T>) -> Self {
        debug_assert!(port1.port_id.kind() != port2.port_id.kind());
        if port1.port_id.kind() == PortKind::Output {
            Self::new(port1, port2)
        } else {
            Self::new(port2, port1)
        }
    }

    /// Returns the source port
    pub fn src(&self) -> GlobalPortId<T> {
        self.src
    }

    /// Returns the destination port
    pub fn dst(&self) -> GlobalPortId<T> {
        self.dst
    }
}
