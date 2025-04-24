mod bindings;
mod events;
#[macro_use]
mod macros;
mod resources;
mod systems;

use std::sync::Mutex;

use bevy::prelude::*;
#[cfg(target_arch = "wasm32")]
use bevy::tasks::{AsyncComputeTaskPool, Task};
use spacetimedb_sdk::{DbContext, Table, TableWithPrimaryKey};

use bindings::*;
use events::{OnInsert, *};

pub use bindings::{
    LobbyRoomTableAccess, Player, create_room as CreateRoom, join_game as JoinGame,
    leave_game as LeaveGame, leave_room as LeaveRoom, mark_cell as MarkCell,
};
pub use resources::*;
pub use systems::*;

#[cfg(not(target_arch = "wasm32"))]
pub fn connect_stdb(app: &mut App) {
    let conn_builder = DbConnection::builder()
        .with_module_name(env!("STDB_MOD_NAME"))
        .with_uri(format!("https://{}", env!("STDB_HOSTNAME")))
        .with_light_mode(true);

    let mut conn = stdb_lifecycle!(app, conn_builder, (OnConnect, OnConnectError, OnDisconnect))
        .build()
        .expect("Failed to establish connection.");

    // run_threaded:    Spawn a thread to process messages in the background.
    // run_async:       Process messages in an async task.
    // frame_tick:      Process messages on the main thread without blocking.
    conn.run_threaded();
    register_callbacks(app, &mut conn);
    app.insert_resource(NetworkConnection::new(conn));
}

#[cfg(not(target_arch = "wasm32"))]
fn register_callbacks(app: &mut App, conn: &mut DbConnection) {
    // Subscription
    stdb_subscribe!(
        app,
        conn,
        "SELECT * FROM lobby_room",            // Query
        OnSubscriptionApplied<Vec<LobbyRoom>>, // Return type / Type to be wrapped by NetworkEvent
        |ctx: &SubscriptionEventContext| {
            let lobby_rooms = ctx.db.lobby_room().iter().collect::<Vec<LobbyRoom>>();
            OnSubscriptionApplied(lobby_rooms)
        }
    );
    // Insert/Delete/Update
    stdb_subscribe!(app, conn, OnInsert<LobbyRoom>);
    stdb_subscribe!(app, conn, OnDelete<LobbyRoom>);
    stdb_subscribe!(app, conn, OnInsert<Game>);
    stdb_subscribe!(app, conn, OnDelete<Game>);
    stdb_subscribe!(app, conn, OnUpdate, Game);

    // Example of subscription with error
    // stdb_event!(
    //     app,
    //     conn,
    //     "SELECT * FROM lobby_room",
    //     OnSubscriptionApplied<Vec<LobbyRoom>>,
    //     |ctx: &SubscriptionEventContext| {
    //         let lobby_rooms = ctx.db.lobby_room().iter().collect::<Vec<LobbyRoom>>();
    //         OnSubscriptionApplied(lobby_rooms)
    //     },
    //     OnSubscriptionError,
    //     |_: &ErrorContext, err| { OnSubscriptionError(err) }
    // );
}

#[cfg(target_arch = "wasm32")]
#[derive(Resource)]
struct PendingConnection(pub Task<DbConnection>);

#[cfg(target_arch = "wasm32")]
pub fn connect_stdb(app: &mut App) {
    let conn_builder = DbConnection::builder()
        .with_module_name(env!("STDB_MOD_NAME"))
        .with_uri(format!("https://{}", env!("STDB_HOSTNAME")))
        .with_light_mode(true);

    let builder = stdb_lifecycle!(app, conn_builder, (OnConnect, OnConnectError, OnDisconnect));

    // Declare all events to be used.
    // We will subscribe to their callbacks after the connection is polled.
    stdb_register_event!(
        app,
        OnSubscriptionApplied<Vec<LobbyRoom>>,
        OnInsert<LobbyRoom>,
        OnDelete<LobbyRoom>,
        OnInsert<Game>,
        OnDelete<Game>,
        OnUpdate<Game>
    );

    let connection_task = AsyncComputeTaskPool::get().spawn(async move {
        builder
            .build()
            .await
            .expect("Failed to establish connection.")
    });

    app.insert_resource(PendingConnection(connection_task));
}

#[cfg(target_arch = "wasm32")]
fn register_callbacks(cmds: &mut Commands, conn: &mut DbConnection) {
    // Subscription
    stdb_subscribe!(
        cmds,
        conn,
        "SELECT * FROM lobby_room",            // Query
        OnSubscriptionApplied<Vec<LobbyRoom>>, // Return type / Type to be wrapped by NetworkEvent
        |ctx: &SubscriptionEventContext| {
            let lobby_rooms = ctx.db.lobby_room().iter().collect::<Vec<LobbyRoom>>();
            OnSubscriptionApplied(lobby_rooms)
        }
    );
    // Insert/Delete/Update
    stdb_subscribe!(cmds, conn, OnInsert<LobbyRoom>);
    stdb_subscribe!(cmds, conn, OnDelete<LobbyRoom>);
    stdb_subscribe!(cmds, conn, OnInsert<Game>);
    stdb_subscribe!(cmds, conn, OnDelete<Game>);
    stdb_subscribe!(cmds, conn, OnUpdate, Game);
}

/// Listens on the EventQueue and writes Bevy events
fn process_network_queue<T: 'static + Send + Sync>(
    maybe_queue: Option<Res<EventQueue<T>>>,
    mut writer: EventWriter<NetworkEvent<T>>,
) {
    if let Some(q) = maybe_queue {
        let queue = q.lock().unwrap();
        if !queue.is_empty() {
            writer.send_batch(queue.try_iter().map(|e| NetworkEvent(e)));
        }
    }
}
