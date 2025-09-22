use crate::circuit_id::{CircuitPortId, PortKind};
use thiserror::Error;

use PortInputState as Pis;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortInputState {
    NoInput,
    Click(CircuitPortId),
    StartConnection(CircuitPortId),
    ProposeConnection(CircuitPortId, CircuitPortId),
    FinalizeConnection(CircuitPortId, CircuitPortId),
}

impl Default for Pis {
    fn default() -> Self {
        Self::NoInput
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CircuitInput {
    state: Pis
}

impl CircuitInput {
    pub fn new() -> Self {
        Self { state: Pis::NoInput }
    }
    
    pub fn state(&self) -> &Pis {
        &self.state
    }

    pub fn start(&mut self, id: CircuitPortId) -> Result<(), ConnectionProposalError> {
        if self.state == Pis::NoInput {
            self.state = Pis::StartConnection(id);
            Ok(())
        } else {
            Err(ConnectionProposalError::StartVariantError)
        }
    }

    pub fn propose(&mut self, id: CircuitPortId) -> Result<(), ConnectionProposalError> {
        match self.state {
            Pis::StartConnection(start) => {
                self.state = Pis::ProposeConnection(start, id);
                Ok(())
            }
            Pis::ProposeConnection(start, _) => {
                self.state = Pis::ProposeConnection(start, id);
                Ok(())
            }
            _ => Err(ConnectionProposalError::ProposeVariantError)
        }
    }

    pub fn finalize(&mut self) -> Result<(), ConnectionProposalError> {
        if let Pis::ProposeConnection(start, end) = self.state {
            if start.port_id.kind() == end.port_id.kind() {
                self.clear();
                Err(ConnectionProposalError::IoMismatch)
            } else if start.port_id.kind() == PortKind::Input {
                self.state = Pis::FinalizeConnection(end, start);
                Ok(())
            } else {
                self.state = Pis::FinalizeConnection(start, end);
                Ok(())
            }
        } else {
            Err(ConnectionProposalError::FinalizeVariantError)
        }
    }

    pub fn click(&mut self, id: CircuitPortId) {
        self.state = Pis::Click(id);
    }

    pub fn clear(&mut self) {
        self.state = Pis::NoInput;
    }
}

#[derive(Debug, Error)]
pub enum ConnectionProposalError {
    #[error("Failed to start connection, NoInput variant required.")]
    StartVariantError,

    #[error("Failed to propose connection, Start(_) or Propose(_, _) variants required for state.")]
    ProposeVariantError,

    #[error("Failed to finalize connection, ProposeConnection(_, _) variant required.")]
    FinalizeVariantError,

    #[error("PortKind mismatch; Must be an input-output pair. Cleared proposal.")]
    IoMismatch,
}
