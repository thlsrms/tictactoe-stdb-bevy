mod bindings;
mod events;
#[macro_use]
mod macros;
mod resources;
mod systems;

use std::sync::Mutex;

use bevy::prelude::*;
use spacetimedb_sdk::{DbContext, Table, TableWithPrimaryKey};

use bindings::*;
use events::{OnInsert, *};

pub use bindings::{
    LobbyRoomTableAccess, Player, create_room as CreateRoom, join_game as JoinGame,
    leave_game as LeaveGame, leave_room as LeaveRoom, mark_cell as MarkCell,
};
pub use resources::*;
pub use systems::*;

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

fn register_callbacks(app: &mut App, conn: &mut DbConnection) {
    // Subscription
    stdb_event!(
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
    stdb_event!(app, conn, OnInsert<LobbyRoom>);
    stdb_event!(app, conn, OnDelete<LobbyRoom>);
    stdb_event!(app, conn, OnInsert<Game>);
    stdb_event!(app, conn, OnDelete<Game>);
    stdb_event!(app, conn, OnUpdate, Game);

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

/// Listens on the EventQueue and writes Bevy events
fn process_network_queue<T: 'static + Send + Sync>(
    queue: Res<EventQueue<T>>,
    mut writer: EventWriter<NetworkEvent<T>>,
) {
    let buf = queue.lock().unwrap();
    if !buf.is_empty() {
        writer.send_batch(buf.try_iter().map(|e| NetworkEvent(e)));
    }
}
