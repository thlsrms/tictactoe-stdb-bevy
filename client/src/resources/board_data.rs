use bevy::prelude::*;

use crate::network::Player;

#[derive(Resource)]
pub struct BoardData {
    pub network_primary: Player,
    pub turn_owner: Player,
    pub x_mask: u16,
    pub o_mask: u16,
    pub result: String,
    pub turn_duration: f32,
    game_id: String,
}

impl BoardData {
    pub fn new(network_primary: Player, game_id: String) -> Self {
        Self {
            network_primary,
            turn_owner: Player::X,
            x_mask: 0,
            o_mask: 0,
            result: "".to_string(),
            turn_duration: 5.,
            game_id,
        }
    }

    pub fn is_primary_turn(&self) -> bool {
        self.turn_owner == self.network_primary
    }

    pub fn id(&self) -> String {
        self.game_id.clone()
    }

    pub fn cell_is_free(&self, cell_mask: u16) -> bool {
        (self.x_mask & cell_mask == 0) && (self.o_mask & cell_mask == 0)
    }

    pub fn set_result_network_primary(&mut self) {
        self.result = "You Won!".to_string()
    }
    pub fn set_result_draw(&mut self) {
        self.result = "It's a DRAW!".to_string()
    }
    pub fn set_result_winner(&mut self, player: &Player) {
        if *player == self.network_primary {
            self.set_result_network_primary();
            return;
        }
        let x_or_o = match player {
            Player::X => "X",
            Player::O => "O",
        };
        self.result = format!("Winner: {x_or_o}");
    }

    pub fn result(&self) -> &str {
        &self.result
    }
}
