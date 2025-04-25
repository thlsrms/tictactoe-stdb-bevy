use std::sync::Mutex;

use bevy::prelude::{Deref, DerefMut, Resource};
use crossbeam_channel::Receiver;
use spacetimedb_sdk::Identity;

use super::DbConnection;

#[derive(Resource)]
pub struct NetworkAuth {
    pub identity: Identity,
    pub authorization: String,
}

#[cfg(target_arch = "wasm32")]
#[derive(Resource)]
pub struct PendingConnection(pub bevy::tasks::Task<DbConnection>);

#[derive(Resource, Deref, DerefMut)]
pub struct NetworkConnection(pub DbConnection);

impl NetworkConnection {
    pub fn new(db_connection: DbConnection) -> Self {
        Self(db_connection)
    }
}

/// Wrapper for sending and receiving network events
#[derive(Resource, Deref, DerefMut)]
pub struct EventQueue<T: Send + Sync>(pub Mutex<Receiver<T>>);
