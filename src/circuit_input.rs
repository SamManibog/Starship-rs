use crate::circuit_id::{CircuitPortId, PortKind, CircuitId};
use thiserror::Error;

use CircuitInputState as Cis;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitInputState {
    NoInput,
    PortClick(CircuitPortId),
    StartConnection(CircuitPortId),
    ProposeConnection(CircuitPortId, CircuitPortId),
    FinalizeConnection(CircuitPortId, CircuitPortId),
    CircuitClick(CircuitId),
}

impl Default for Cis {
    fn default() -> Self {
        Self::NoInput
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CircuitInput {
    state: Cis
}

impl CircuitInput {
    pub fn new() -> Self {
        Self { state: Cis::NoInput }
    }
    
    pub fn state(&self) -> &Cis {
        &self.state
    }

    pub fn start(&mut self, id: CircuitPortId) -> Result<(), ConnectionProposalError> {
        if self.state == Cis::NoInput {
            self.state = Cis::StartConnection(id);
            Ok(())
        } else {
            Err(ConnectionProposalError::StartVariantError)
        }
    }

    pub fn propose(&mut self, id: CircuitPortId) -> Result<(), ConnectionProposalError> {
        match self.state {
            Cis::StartConnection(start) => {
                self.state = Cis::ProposeConnection(start, id);
                Ok(())
            }
            Cis::ProposeConnection(start, _) => {
                self.state = Cis::ProposeConnection(start, id);
                Ok(())
            }
            _ => Err(ConnectionProposalError::ProposeVariantError)
        }
    }

    pub fn finalize(&mut self) -> Result<(), ConnectionProposalError> {
        if let Cis::ProposeConnection(start, end) = self.state {
            if start.port_id.kind() == end.port_id.kind() {
                self.clear();
                Err(ConnectionProposalError::IoMismatch)
            } else if start.port_id.kind() == PortKind::Input {
                self.state = Cis::FinalizeConnection(end, start);
                Ok(())
            } else {
                self.state = Cis::FinalizeConnection(start, end);
                Ok(())
            }
        } else {
            Err(ConnectionProposalError::FinalizeVariantError)
        }
    }

    pub fn click(&mut self, id: CircuitPortId) {
        self.state = Cis::PortClick(id);
    }

    pub fn circuit_click(&mut self, id: CircuitId) {
        self.state = Cis::CircuitClick(id);
    }

    pub fn clear(&mut self) {
        self.state = Cis::NoInput;
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
