use spacetimedb::{client_visibility_filter, Filter, Identity, ReducerContext, Table};

use crate::game_turn_scheduler::set_turn_expiration_schedule;
use crate::types::{GameState, Player};

// RLS
#[client_visibility_filter]
const GAME_ACCESS_FILTER: Filter =
    Filter::Sql("SELECT * FROM game WHERE x_player = :sender OR o_player = :sender");
#[spacetimedb::table(name = game, public)]
#[derive(Clone)]
pub struct Game {
    #[primary_key]
    pub id: String, // Investigate the possibility to use GameId with FilterableValue
    pub x_player: Identity,
    pub o_player: Identity,
    pub turn_owner: Player,
    pub x_mask: u16,
    pub o_mask: u16,
    pub state: GameState,
    pub turn: u8,
    pub time_expired: bool,
}

#[spacetimedb::reducer]
pub fn mark_cell(ctx: &ReducerContext, game_id: String, cell: u16) -> Result<(), String> {
    let Some(mut game) = ctx.db.game().id().find(game_id.clone()) else {
        return Err("Invalid Game '{game_id}'".to_string());
    };
    if !game.game_in_progress() {
        return Err("Game '{game_id}' not in progress.".to_string());
    }
    if !game.validate_turn_owner(ctx.sender) {
        return Err("Not your turn".to_string());
    }
    game.toggle_cell(cell)?;
    game.result_or_next_turn();

    // Schedule Turn Expiration
    set_turn_expiration_schedule(ctx, game_id, game.turn);

    ctx.db.game().id().update(game);
    Ok(())
}

#[spacetimedb::reducer]
pub fn leave_game(ctx: &ReducerContext, game_id: String) {
    let Some(game) = ctx.db.game().id().find(game_id) else {
        return;
    };
    if (ctx.sender == game.x_player) || (ctx.sender == game.o_player) {
        ctx.db.game().delete(game);
    }
}

impl Game {
    pub fn new(x_player: Identity, o_player: Identity, id: String) -> Self {
        Self {
            id,
            x_player,
            o_player,
            turn_owner: Player::X,
            x_mask: 0,
            o_mask: 0,
            state: GameState::InProgress,
            turn: 0,
            time_expired: false,
        }
    }

    pub fn game_in_progress(&self) -> bool {
        // GameState InProgress
        matches!(self.state, GameState::InProgress)
    }

    pub fn validate_turn_owner(&self, player: Identity) -> bool {
        match self.turn_owner {
            Player::X => self.x_player == player,
            Player::O => self.o_player == player,
        }
    }

    pub fn result_or_next_turn(&mut self) {
        /*
         * Bitboard:
         * Map each board cell to one of nine bits (positions 0 to 8):
         * (0,0) -> bit 0, (1,0) -> bit 1, (2,0) -> bit 2, (0,1) -> bit 3, ..., (2,2) -> bit 8.
         */
        const WINNING_MASKS: [u16; 8] = [
            0b000_000_111, // row 0
            0b000_111_000, // row 1
            0b111_000_000, // row 2
            0b001_001_001, // col 0
            0b010_010_010, // col 1
            0b100_100_100, // col 2
            0b100_010_001, // main diagonal
            0b001_010_100, // anti-diagonal
        ];

        let curr_player_cells = match self.turn_owner {
            Player::X => self.x_mask,
            Player::O => self.o_mask,
        };

        let mut any_open = false;
        for &mask in WINNING_MASKS.iter() {
            // Check for a win on this mask
            if (curr_player_cells & mask) == mask {
                self.state = GameState::Winner(self.turn_owner);
                return;
            }

            // Check if the line is not yet fully blocked (i.e. it's still open for a win)
            // Note: We use logical OR so that once any_open is true, it stays true.
            any_open |= !((mask & self.x_mask != 0) && (mask & self.o_mask != 0));
        }

        if any_open {
            self.next_turn();
        } else {
            // Every mask is blocked, it's a draw.
            self.state = GameState::Draw;
        }
    }

    fn next_turn(&mut self) {
        self.time_expired = false;
        self.turn_owner = match self.turn_owner {
            Player::X => Player::O,
            Player::O => Player::X,
        };
        self.turn += 1;
    }

    pub fn turn_expired(&mut self) {
        self.turn_owner = match self.turn_owner {
            Player::X => Player::O,
            Player::O => Player::X,
        };
        self.time_expired = true;
    }

    pub fn toggle_cell(&mut self, cell: u16) -> Result<(), String> {
        // Check if cell if free
        if !((self.x_mask & cell == 0) && (self.o_mask & cell == 0)) {
            return Err("Cell already taken".to_string());
        }

        match self.turn_owner {
            Player::X => self.x_mask |= cell,
            Player::O => self.o_mask |= cell,
        };
        Ok(())
    }
}
