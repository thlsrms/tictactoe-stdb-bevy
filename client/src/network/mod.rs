mod bindings;
mod events;
#[macro_use]
mod macros;
mod resources;
mod systems;

use std::sync::Mutex;

use bevy::prelude::*;
#[cfg(target_arch = "wasm32")]
use bevy::tasks::AsyncComputeTaskPool;
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
        .with_uri(format!("https://{}", env!("STDB_HOST")))
        .with_light_mode(true);

    let conn_builder =
        stdb_lifecycle!(app, conn_builder, (OnConnect, OnConnectError, OnDisconnect));

    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut conn = conn_builder
            .build()
            .expect("Failed to establish connection.");
        conn.run_threaded();
        register_callbacks(app, &mut conn);
        app.insert_resource(NetworkConnection::new(conn));
    }

    #[cfg(target_arch = "wasm32")]
    {
        // The web version needs to poll the connection
        let connection_task = AsyncComputeTaskPool::get().spawn(async move {
            conn_builder
                .build()
                .await
                .expect("Failed to establish connection.")
        });
        app.insert_resource(PendingConnection(connection_task));

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
    }
}

#[cfg(not(target_arch = "wasm32"))]
use bevy::prelude::App as Context;
#[cfg(target_arch = "wasm32")]
use bevy::prelude::Commands as Context;

fn register_callbacks(ctx: &mut Context, conn: &mut DbConnection) {
    // Subscription
    stdb_subscribe!(
        ctx,
        conn,
        "SELECT * FROM lobby_room",            // Query
        OnSubscriptionApplied<Vec<LobbyRoom>>, // Return type / Type to be wrapped by NetworkEvent
        |ctx: &SubscriptionEventContext| {
            let lobby_rooms = ctx.db.lobby_room().iter().collect::<Vec<LobbyRoom>>();
            OnSubscriptionApplied(lobby_rooms)
        }
    );
    // Insert/Delete/Update
    stdb_subscribe!(ctx, conn, OnInsert<LobbyRoom>);
    stdb_subscribe!(ctx, conn, OnDelete<LobbyRoom>);
    stdb_subscribe!(ctx, conn, OnInsert<Game>);
    stdb_subscribe!(ctx, conn, OnDelete<Game>);
    stdb_subscribe!(ctx, conn, OnUpdate, Game);

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
