static mut NEXT_ID: usize = 0;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CircuitId {
    id: usize
}

impl CircuitId {
    ///Creates a new circuit id
    ///You must ensure there are no race conditions involving the simultaneous creation of other CircuitId's
    pub unsafe fn new() -> Self {
        let id = unsafe { NEXT_ID };
        unsafe { NEXT_ID += 1 };
        Self {
            id
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }
}

impl std::fmt::Debug for CircuitId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CircuitId: {}", self.id)
    }
}
