use spacetimedb::rand::RngCore;
use spacetimedb::{Identity, ReducerContext, Table};

use crate::game_table::{game as _, Game};
use crate::game_turn_scheduler::set_turn_expiration_schedule;

#[spacetimedb::table(name = lobby_room, public)]
pub struct LobbyRoom {
    #[auto_inc]
    #[primary_key]
    pub id: u32,
    #[unique]
    pub game_id: String,
    #[unique]
    pub owner: Identity,
}

#[spacetimedb::reducer]
pub fn create_room(ctx: &ReducerContext) {
    if ctx.db.lobby_room().owner().find(ctx.sender).is_some() {
        log::warn!("{} is trying to own two games", ctx.sender);
        return;
    }

    // Base58 alphabet excluding ambiguous characters (0, O, I, l)
    const BASE58_ALPHABET: [char; 58] = [
        '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J',
        'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c',
        'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v',
        'w', 'x', 'y', 'z',
    ];

    let mut rng = ctx.rng();
    let mut bytes: Vec<u8> = vec![0; 8];
    rng.fill_bytes(&mut bytes);
    let id: String = bytes
        .iter()
        .map(|&b| BASE58_ALPHABET[b as usize % BASE58_ALPHABET.len()])
        .collect();

    log::info!("Game ID generated {id}");
    ctx.db.lobby_room().insert(LobbyRoom {
        id: 0,
        game_id: id.clone(),
        owner: ctx.sender,
    });
}

#[spacetimedb::reducer]
pub fn join_game(ctx: &ReducerContext, room_id: u32) {
    if let Some(room) = ctx.db.lobby_room().id().find(room_id) {
        if room.owner == ctx.sender {
            log::warn!("Room owner trying to join his own room.");
            return;
        }
        ctx.db
            .game()
            .insert(Game::new(room.owner, ctx.sender, room.game_id.clone()));

        // Schedule Turn Expiration
        set_turn_expiration_schedule(ctx, room.game_id.clone(), 0);

        ctx.db.lobby_room().delete(room);
    };
}

#[spacetimedb::reducer]
pub fn leave_room(ctx: &ReducerContext) {
    ctx.db.lobby_room().owner().delete(ctx.sender);
}
