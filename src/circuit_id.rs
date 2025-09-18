static mut NEXT_ID: u32 = 0;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CircuitId {
    id: u32
}

impl CircuitId {
    ///Creates a new circuit id
    ///You must ensure there are no race conditions involving the simultaneous creation of other CircuitIds
    pub unsafe fn new() -> Self {
        let id = unsafe { NEXT_ID };
        unsafe { NEXT_ID += 1 };
        Self {
            id
        }
    }

    pub fn raw(&self) -> u32 {
        self.id
    }
}

impl std::fmt::Debug for CircuitId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CircuitId: {}", self.id)
    }
}

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
    pub fn new(src: CircuitPortId, dst: CircuitPortId) -> Self {
        assert!(src.port_id().kind() == PortKind::Output);
        assert!(dst.port_id().kind() == PortKind::Input);
        Self {
            src,
            dst,
        }
    }

    pub fn src(&self) -> CircuitPortId {
        self.src
    }

    pub fn dst(&self) -> CircuitPortId {
        self.dst
    }
}
