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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

///The identifier for a port on a specific circuit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CircuitPortId {
    pub circuit_id: CircuitId,
    pub port_id: PortId,
}

impl CircuitPortId {
    pub fn new(circuit_id: CircuitId, port_id: PortId) -> Self {
        Self {
            circuit_id,
            port_id
        }
    }

    pub fn circuit_id(&self) -> CircuitId {
        self.circuit_id
    }

    pub fn port_id(&self) -> PortId {
        self.port_id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionId {
    src: CircuitPortId,
    dst: CircuitPortId,
}

impl ConnectionId {
    /// Creates a connection Id where source and destination are specified
    pub fn new(src: CircuitPortId, dst: CircuitPortId) -> Self {
        debug_assert!(src.port_id().kind() == PortKind::Output);
        debug_assert!(dst.port_id().kind() == PortKind::Input);
        Self {
            src,
            dst,
        }
    }

    /// Creates a connection id, automatically determinging the source and destination
    pub fn new_auto(port1: CircuitPortId, port2: CircuitPortId) -> Self {
        debug_assert!(port1.port_id().kind() != port2.port_id().kind());
        if port1.port_id.kind() == PortKind::Output {
            Self::new(port1, port2)
        } else {
            Self::new(port2, port1)
        }
    }

    /// Returns the source port
    pub fn src(&self) -> CircuitPortId {
        self.src
    }

    /// Returns the destination port
    pub fn dst(&self) -> CircuitPortId {
        self.dst
    }
}
