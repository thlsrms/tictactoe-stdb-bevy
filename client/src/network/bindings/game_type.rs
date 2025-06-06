// THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE
// WILL NOT BE SAVED. MODIFY TABLES IN YOUR MODULE SOURCE CODE INSTEAD.

#![allow(unused, clippy::all)]
use spacetimedb_sdk::__codegen::{self as __sdk, __lib, __sats, __ws};

use super::game_state_type::GameState;
use super::player_type::Player;

#[derive(__lib::ser::Serialize, __lib::de::Deserialize, Clone, PartialEq, Debug)]
#[sats(crate = __lib)]
pub struct Game {
    pub id: String,
    pub x_player: __sdk::Identity,
    pub o_player: __sdk::Identity,
    pub turn_owner: Player,
    pub x_mask: u16,
    pub o_mask: u16,
    pub state: GameState,
    pub turn: u8,
    pub time_expired: bool,
}

impl __sdk::InModule for Game {
    type Module = super::RemoteModule;
}
