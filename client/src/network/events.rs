use bevy::prelude::{Deref, DerefMut, Event};
use spacetimedb_sdk::{Error as StdbError, Identity};

use super::NetworkAuth;

/// Wrapper to help us filter network events
#[derive(Event, Deref, DerefMut)]
pub struct NetworkEvent<E>(pub E);

// Event definitions
#[derive(Deref, DerefMut)]
pub struct OnInsert<Row>(pub Row);
#[derive(Deref, DerefMut)]
pub struct OnDelete<Row>(pub Row);
pub struct OnUpdate<Row> {
    pub old: Row,
    pub new: Row,
}
#[derive(Deref, DerefMut)]
pub struct OnSubscriptionApplied<D>(pub D);
#[allow(dead_code)]
#[derive(Deref, DerefMut)]
pub struct OnSubscriptionError(pub StdbError);

#[derive(Deref, DerefMut)]
pub struct OnConnect(pub NetworkAuth);

impl OnConnect {
    pub fn new(identity: Identity, token: &str) -> Self {
        Self(NetworkAuth {
            identity,
            authorization: token.into(),
        })
    }
}

#[allow(dead_code)]
#[derive(Deref, DerefMut)]
pub struct OnConnectError(pub StdbError);

impl OnConnectError {
    pub fn new(error: StdbError) -> Self {
        Self(error)
    }
}

#[allow(dead_code)]
#[derive(Deref, DerefMut)]
pub struct OnDisconnect(pub Option<StdbError>);

impl OnDisconnect {
    pub fn new(error: Option<StdbError>) -> Self {
        Self(error)
    }
}
