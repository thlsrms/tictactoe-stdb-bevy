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
        .with_uri(env!("STDB_HOST"))
        .with_light_mode(true);

    // Register the events for the lifecycle callbacks: OnConnect, OnConnectError, OnDisconnect
    let conn_builder = stdb_lifecycle_events!(app, conn_builder);

    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut conn = conn_builder
            .build()
            .expect("Failed to establish connection.");
        register_callbacks(app, &mut conn);
        conn.run_threaded();
        app.insert_resource(NetworkConnection::new(conn));
    }

    #[cfg(target_arch = "wasm32")]
    {
        // Re-exported browser storage api wrappers
        // use spacetimedb_sdk::credentials::{LocalStorage, SessionStorage, Storage};
        use spacetimedb_sdk::credentials::cookies::Cookie;
        let conn_builder = if let Ok(Some(token)) = Cookie::get("tictactoe_auth") {
            conn_builder.with_token(token)
        } else {
            conn_builder
        };

        // The web version needs to poll the connection
        let connection_task = AsyncComputeTaskPool::get().spawn(async move {
            conn_builder
                .build()
                .await
                .expect("Failed to establish connection.")
        });
        app.insert_resource(PendingConnection(connection_task));

        // Declare all events to be used.
        // We need to subscribe to their callbacks after the connection is polled.
        // OnEvent<Table>
        stdb_register_event!(
            app,
            OnSubApplied<Vec<LobbyRoom>>,
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
        "SELECT * FROM lobby_room", // Query
        Vec<LobbyRoom>,             // Return type / Type to be wrapped by Stdb<OnSubApplied<T>>
        |ctx| { ctx.db.lobby_room().iter().collect::<Vec<LobbyRoom>>() },
        // Should subscribe to `on_error`? Stdb(OnSubError<T>(error))
        false // can be omitted for false
    );
    // Insert/Delete/Update -> Are wrapped by: Stdb<OnInsert | OnDelete | OnUpdate <Table>>
    stdb_subscribe!(ctx, conn, insert, LobbyRoom);
    stdb_subscribe!(ctx, conn, delete, LobbyRoom);
    stdb_subscribe!(ctx, conn, insert, Game);
    stdb_subscribe!(ctx, conn, delete, Game);
    stdb_subscribe!(ctx, conn, update, Game);
}

/// Listens on the EventQueue and writes Bevy events
fn process_network_queue<T: 'static + Send + Sync>(
    maybe_queue: Option<Res<EventQueue<T>>>,
    mut writer: EventWriter<Stdb<T>>,
) {
    if let Some(q) = maybe_queue {
        let queue = q.lock().unwrap();
        if !queue.is_empty() {
            writer.write_batch(queue.try_iter().map(|e| Stdb(e)));
        }
    }
}
