mod game_table;
mod game_turn_scheduler;
mod lobby_room_table;
mod types;

use log::info;
use spacetimedb::{ReducerContext, Table};

pub use game_table::*;
pub use lobby_room_table::*;

#[spacetimedb::reducer(init)]
pub fn init(_ctx: &ReducerContext) {
    // Called when the module is initially published
}

#[spacetimedb::reducer(client_connected)]
pub fn identity_connected(ctx: &ReducerContext) {
    // Called everytime a new client connects
    info!("!!!!!! Client connected {}", ctx.sender);
}

#[spacetimedb::reducer(client_disconnected)]
pub fn identity_disconnected(ctx: &ReducerContext) {
    // Delete any open lobbies the client owned
    if let Some(room) = ctx.db.lobby_room().owner().find(ctx.sender) {
        ctx.db.lobby_room().delete(room);
    }

    // We delete any game that the client was part of:
    if let Some(game) = ctx
        .db
        .game()
        .iter()
        .find(|t| (t.o_player == ctx.sender) || (t.x_player == ctx.sender))
    {
        ctx.db.game().delete(game);
    }
}
