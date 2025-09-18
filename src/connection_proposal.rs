use crate::circuit_id::{CircuitPortId, PortKind};
use thiserror::Error;

use ConnectionProposalState as Cps;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionProposalState {
    NotStarted,
    Started(CircuitPortId),
    Proposed(CircuitPortId, CircuitPortId),
    Finalized(CircuitPortId, CircuitPortId),
}

impl ConnectionProposalState {
    pub fn kind(&self) -> ConnectionProposalKind {
        match self {
            Self::NotStarted => ConnectionProposalKind::NotStarted,
            Self::Started(_) => ConnectionProposalKind::Started,
            Self::Proposed(_, _) => ConnectionProposalKind::Proposed,
            Self::Finalized(_, _) => ConnectionProposalKind::Finalized,
        }
    }
}

impl Default for Cps {
    fn default() -> Self {
        Self::NotStarted
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ConnectionProposal {
    state: Cps
}

impl ConnectionProposal {
    pub fn new() -> Self {
        Self { state: Cps::NotStarted }
    }
    
    pub fn kind(&self) -> ConnectionProposalKind {
        self.state.kind()
    }

    pub fn state(&self) -> &Cps {
        &self.state
    }

    pub fn start(&mut self, id: CircuitPortId) -> Result<(), ConnectionProposalError> {
        if self.state == Cps::NotStarted {
            self.state = Cps::Started(id);
            Ok(())
        } else {
            Err(ConnectionProposalError::KindMismatch(ConnectionProposalKind::NotStarted))
        }
    }

    pub fn end(&mut self, id: CircuitPortId) -> Result<(), ConnectionProposalError> {
        match self.state {
            Cps::Started(start) => {
                self.state = Cps::Proposed(start, id);
                Ok(())
            }
            Cps::Proposed(start, _) => {
                self.state = Cps::Proposed(start, id);
                Ok(())
            }
            _ => Err(ConnectionProposalError::KindMismatch(ConnectionProposalKind::Started))
        }
    }

    pub fn finalize(&mut self) -> Result<(), ConnectionProposalError> {
        if let Cps::Proposed(start, end) = self.state {
            if start.port_id.kind() == end.port_id.kind() {
                self.cancel();
                Err(ConnectionProposalError::IoMismatch)
            } else if start.port_id.kind() == PortKind::Input {
                self.state = Cps::Finalized(end, start);
                Ok(())
            } else {
                self.state = Cps::Finalized(start, end);
                Ok(())
            }
        } else {
            Err(ConnectionProposalError::KindMismatch(ConnectionProposalKind::Proposed))
        }
    }

    pub fn cancel(&mut self) {
        self.state = Cps::NotStarted
    }
}

#[derive(Debug, Error)]
pub enum ConnectionProposalError {
    #[error("Wrong state kind; {0} required.")]
    KindMismatch(ConnectionProposalKind),

    #[error("PortKind mismatch; Must be an input-output pair. Cleared proposal.")]
    IoMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionProposalKind {
    NotStarted,
    Started,
    Proposed,
    Finalized,
}

impl std::fmt::Display for ConnectionProposalKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::NotStarted => "NotStarted",
            Self::Started => "Started",
            Self::Proposed => "Proposed",
            Self::Finalized => "Finalized"
        })
    }
}
