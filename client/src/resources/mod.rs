mod board_data;
mod fonts;

pub use board_data::*;
pub use fonts::*;

use std::collections::HashMap;

use bevy::ecs::system::SystemId;
use bevy::prelude::{Deref, DerefMut, FromWorld, Resource, World};

use crate::ui;

#[derive(Resource, Deref, DerefMut)]
pub struct BoardSystems(HashMap<String, SystemId>);

impl FromWorld for BoardSystems {
    fn from_world(world: &mut World) -> Self {
        let mut map = HashMap::new();
        map.insert("clear_board".into(), world.register_system(ui::clear_board));
        Self(map)
    }
}
