use bevy::prelude::*;

use crate::Player;

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

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GameResult {
    Draw,
    Winner(Player),
}

impl AsRef<str> for GameResult {
    fn as_ref(&self) -> &str {
        match self {
            GameResult::Draw => "It's a DRAW!",
            GameResult::Winner(player) => match player {
                crate::Player::X => "Winner: X!",
                crate::Player::O => "Winner: O!",
            },
        }
    }
}

#[derive(Resource)]
pub struct BoardData {
    turn_owner: Player,
    x_mask: u16,
    o_mask: u16,
    result: Option<GameResult>,
}

impl BoardData {
    pub fn new() -> Self {
        Self {
            turn_owner: Player::X,
            x_mask: 0,
            o_mask: 0,
            result: None,
        }
    }

    pub fn turn_owner(&self) -> Player {
        self.turn_owner
    }

    pub fn result(&self) -> Option<GameResult> {
        self.result
    }

    pub fn next_turn(&mut self) {
        self.turn_owner = match self.turn_owner {
            Player::X => Player::O,
            Player::O => Player::X,
        };
    }

    pub fn mark_cell(&mut self, cell_mask: u16) {
        match self.turn_owner {
            Player::X => self.x_mask |= cell_mask,
            Player::O => self.o_mask |= cell_mask,
        }
        self.check_winner();
    }

    /// Check if the current player has a winning combination.
    /// Uses precomputed winning masks to determine a win.
    pub fn check_winner(&mut self) {
        let curr_player_mask = {
            match self.turn_owner {
                Player::X => self.x_mask,
                Player::O => self.o_mask,
            }
        };

        let match_won = WINNING_MASKS
            .iter()
            .any(|&mask| (curr_player_mask & mask) == mask);

        if match_won {
            self.result = Some(GameResult::Winner(self.turn_owner()));
        } else if self.has_potential_win() {
            self.next_turn();
        } else {
            self.result = Some(GameResult::Draw);
        }
    }

    /// Check if there's any line still potentially open for a win.
    fn has_potential_win(&self) -> bool {
        for &mask in &WINNING_MASKS {
            // The mask is blocked if both players have occupied a cell in the mask
            if !((mask & self.x_mask != 0) && (mask & self.o_mask != 0)) {
                return true;
            }
        }
        false
    }

    pub fn cell_is_free(&self, cell_mask: u16) -> bool {
        (self.x_mask & cell_mask == 0) && (self.o_mask & cell_mask == 0)
    }
}
