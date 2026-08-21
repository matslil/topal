//! Capability-authorized data-transfer protocols and reference backends.

use std::collections::VecDeque;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

pub mod database;
pub mod device;
pub mod file_store;
pub mod firewall;
pub mod framing;
pub mod host;
pub mod i2c;
pub mod native;
pub mod network;
pub mod operation;
pub mod region;
pub mod store;
pub mod transport;
pub mod view;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn fresh_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

/// Stable semantic identity of an endpoint.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EndpointId(u64);

/// Stable application-service identity, independent of an address or endpoint.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ServiceId(u64);

impl ServiceId {
    /// Allocates an identity in the embedding process.
    #[must_use]
    pub fn allocate() -> Self {
        Self(fresh_id())
    }
}

/// Authority to use exactly one endpoint.
#[derive(Clone)]
pub struct EndpointCapability {
    identity: EndpointId,
    authority: Arc<()>,
}

impl fmt::Debug for EndpointCapability {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EndpointCapability")
            .field("identity", &self.identity)
            .finish_non_exhaustive()
    }
}

impl EndpointCapability {
    /// Returns the stable endpoint identity without exposing authority internals.
    #[must_use]
    pub const fn identity(&self) -> EndpointId {
        self.identity
    }

    fn authorizes(&self, endpoint: EndpointId, authority: &Arc<()>) -> bool {
        self.identity == endpoint && Arc::ptr_eq(&self.authority, authority)
    }
}

/// Legal state of the reference message protocol.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndpointState {
    /// Messages may be sent and received.
    Open,
    /// No further operation may be submitted.
    Closed,
}

/// Typed failures shared by the endpoint foundation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndpointFailure {
    /// The supplied capability does not authorize this endpoint.
    DeniedAuthority,
    /// The endpoint is closed.
    Closed,
    /// The bounded receiver queue cannot accept another message.
    Exhausted,
    /// No message is currently available.
    Pending,
    /// The local reference endpoint became unavailable.
    Unavailable,
}

#[derive(Debug)]
struct State<Message> {
    protocol: EndpointState,
    inbox: VecDeque<Message>,
}

/// One side of a deterministic, bounded, in-memory message endpoint pair.
#[derive(Debug)]
pub struct LocalEndpoint<Message> {
    identity: EndpointId,
    authority: Arc<()>,
    own: Arc<Mutex<State<Message>>>,
    peer: Arc<Mutex<State<Message>>>,
    capacity: usize,
}

impl<Message> LocalEndpoint<Message> {
    /// Creates a connected endpoint pair and their non-ambient capabilities.
    #[must_use]
    pub fn pair(capacity: usize) -> ((Self, EndpointCapability), (Self, EndpointCapability)) {
        let left_state = Arc::new(Mutex::new(State {
            protocol: EndpointState::Open,
            inbox: VecDeque::new(),
        }));
        let right_state = Arc::new(Mutex::new(State {
            protocol: EndpointState::Open,
            inbox: VecDeque::new(),
        }));
        let left_id = EndpointId(fresh_id());
        let right_id = EndpointId(fresh_id());
        let left_authority = Arc::new(());
        let right_authority = Arc::new(());
        let left = Self {
            identity: left_id,
            authority: Arc::clone(&left_authority),
            own: Arc::clone(&left_state),
            peer: Arc::clone(&right_state),
            capacity,
        };
        let right = Self {
            identity: right_id,
            authority: Arc::clone(&right_authority),
            own: right_state,
            peer: left_state,
            capacity,
        };
        (
            (
                left,
                EndpointCapability {
                    identity: left_id,
                    authority: left_authority,
                },
            ),
            (
                right,
                EndpointCapability {
                    identity: right_id,
                    authority: right_authority,
                },
            ),
        )
    }

    fn authorize(&self, capability: &EndpointCapability) -> Result<(), EndpointFailure> {
        capability
            .authorizes(self.identity, &self.authority)
            .then_some(())
            .ok_or(EndpointFailure::DeniedAuthority)
    }

    /// Returns this endpoint's protocol state.
    ///
    /// # Errors
    ///
    /// Returns `DeniedAuthority` for the wrong capability or `Unavailable` if
    /// the reference endpoint cannot safely access its state.
    pub fn state(&self, capability: &EndpointCapability) -> Result<EndpointState, EndpointFailure> {
        self.authorize(capability)?;
        Ok(self
            .own
            .lock()
            .map_err(|_| EndpointFailure::Unavailable)?
            .protocol)
    }

    /// Sends one application message while preserving its boundary.
    ///
    /// # Errors
    ///
    /// Returns the precise authority, protocol, capacity, or local
    /// availability failure without consuming `message` into the peer queue.
    pub fn send(
        &self,
        capability: &EndpointCapability,
        message: Message,
    ) -> Result<(), EndpointFailure> {
        self.authorize(capability)?;
        if self
            .own
            .lock()
            .map_err(|_| EndpointFailure::Unavailable)?
            .protocol
            == EndpointState::Closed
        {
            return Err(EndpointFailure::Closed);
        }
        let mut peer = self.peer.lock().map_err(|_| EndpointFailure::Unavailable)?;
        if peer.protocol == EndpointState::Closed {
            return Err(EndpointFailure::Closed);
        }
        if peer.inbox.len() >= self.capacity {
            return Err(EndpointFailure::Exhausted);
        }
        peer.inbox.push_back(message);
        Ok(())
    }

    /// Receives one complete application message.
    ///
    /// # Errors
    ///
    /// Returns the precise authority, protocol, pending, or local availability
    /// failure.
    pub fn receive(&self, capability: &EndpointCapability) -> Result<Message, EndpointFailure> {
        self.authorize(capability)?;
        let mut own = self.own.lock().map_err(|_| EndpointFailure::Unavailable)?;
        own.inbox.pop_front().ok_or_else(|| {
            if own.protocol == EndpointState::Closed {
                EndpointFailure::Closed
            } else {
                EndpointFailure::Pending
            }
        })
    }

    /// Closes this endpoint. Queued messages remain consumable.
    ///
    /// # Errors
    ///
    /// Returns `DeniedAuthority` for the wrong capability or `Unavailable` if
    /// the reference endpoint cannot safely access its state.
    pub fn close(&self, capability: &EndpointCapability) -> Result<(), EndpointFailure> {
        self.authorize(capability)?;
        self.own
            .lock()
            .map_err(|_| EndpointFailure::Unavailable)?
            .protocol = EndpointState::Closed;
        Ok(())
    }
}

/// Typed request/reply service independent of its binding.
pub trait RequestReply<Request> {
    /// Successful reply type.
    type Reply;
    /// Service-specific typed failure.
    type Failure;

    /// Applies one request.
    ///
    /// # Errors
    ///
    /// Returns a service-specific typed failure when the request cannot be
    /// applied.
    fn request(&mut self, request: Request) -> Result<Self::Reply, Self::Failure>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pair_preserves_message_boundaries_and_backpressure() {
        let ((left, left_cap), (right, right_cap)) = LocalEndpoint::pair(1);
        left.send(&left_cap, vec![1, 2, 3]).unwrap();
        assert_eq!(
            left.send(&left_cap, vec![4]),
            Err(EndpointFailure::Exhausted)
        );
        assert_eq!(right.receive(&right_cap).unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn capability_is_confined_to_its_endpoint() {
        let ((left, left_cap), (right, right_cap)) = LocalEndpoint::<u8>::pair(1);
        assert_eq!(
            left.send(&right_cap, 1),
            Err(EndpointFailure::DeniedAuthority)
        );
        left.send(&left_cap, 2).unwrap();
        assert_eq!(right.receive(&right_cap), Ok(2));
    }

    #[test]
    fn close_is_a_protocol_transition() {
        let ((left, left_cap), _) = LocalEndpoint::<u8>::pair(1);
        left.close(&left_cap).unwrap();
        assert_eq!(left.state(&left_cap), Ok(EndpointState::Closed));
        assert_eq!(left.send(&left_cap, 1), Err(EndpointFailure::Closed));
    }
}
